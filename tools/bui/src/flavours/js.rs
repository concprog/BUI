//! JavaScript and TypeScript: React, Svelte or BUI's own DOM-like API, on Bun, npm or pnpm, with
//! Biome for lint and format and TypeScript's strict mode.

use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{File, Flavour, Step, Vars, read, render, step};
use crate::check::{self, Check, Fix};
use crate::project::{Framework, Language, PackageManager};
use crate::registry;
use crate::sys::Cmd;
use crate::tools::Tool;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsApp {
    pub framework: Framework,
    pub language: Language,
    pub pm: PackageManager,
}

impl PackageManager {
    const fn bin(self) -> &'static str {
        match self {
            Self::Bun => "bun",
            Self::Npm => "npm",
            Self::Pnpm => "pnpm",
        }
    }

    /// `bun add` / `npm install` / `pnpm add`, with the packages at their newest versions.
    fn add(self, dir: &Path, dev: bool, packages: &[&str]) -> Cmd {
        let mut args: Vec<&str> = vec![match self {
            Self::Npm => "install",
            Self::Bun | Self::Pnpm => "add",
        }];
        if dev {
            args.push(if self == Self::Npm { "--save-dev" } else { "--dev" });
        }
        args.extend_from_slice(packages);
        Cmd::new(self.bin(), args).in_dir(dir)
    }

    fn run(self, dir: &Path, script: &str) -> Cmd {
        Cmd::new(self.bin(), ["run", script]).in_dir(dir)
    }

    /// Everything the lock file names, at the newest version the ranges allow.
    fn update(self, dir: &Path) -> Cmd {
        Cmd::new(self.bin(), ["update"]).in_dir(dir)
    }
}

impl JsApp {
    const fn typed(&self) -> bool {
        matches!(self.language, Language::TypeScript)
    }

    /// The script the app starts from.
    const fn entry(&self) -> &'static str {
        match (self.framework, self.language) {
            (Framework::React, Language::TypeScript) => "src/main.tsx",
            (Framework::React, Language::JavaScript) => "src/main.jsx",
            (Framework::Svelte, _) | (Framework::Vanilla, Language::JavaScript) => "src/main.js",
            (Framework::Vanilla, Language::TypeScript) => "src/main.ts",
        }
    }

    /// Run by `tsx` (TypeScript and JSX on Node): in CommonJS mode, as Node cannot import the named
    /// exports of BUI's CommonJS packages from an ES module.
    fn uses_tsx(&self) -> bool {
        self.pm != PackageManager::Bun && self.framework != Framework::Svelte && (self.typed() || self.framework == Framework::React)
    }

    /// The command that runs the entry, as `bun`, `tsx` or `node` do, with the Svelte loader where needed.
    fn runner(&self, watch: bool) -> String {
        let entry = self.entry();
        let flag = |on: &'static str| if watch { on } else { "" };
        match self.framework {
            // @blender-ui/svelte's loader does not work under Bun (its compiler call fails there): Node runs it
            Framework::Svelte => format!("node --import @blender-ui/svelte/register {}{entry}", flag("--watch ")),
            _ if self.pm == PackageManager::Bun => format!("bun {}{entry}", flag("--watch ")),
            _ if self.uses_tsx() => format!("tsx {}{entry}", flag("watch ")),
            _ => format!("node {}{entry}", flag("--watch ")),
        }
    }

    fn package_json(&self, name: &str) -> String {
        let mut scripts = serde_json::Map::new();
        let mut put = |k: &str, v: String| {
            scripts.insert(k.into(), json!(v));
        };
        put("dev", self.runner(true));
        put("start", self.runner(false));
        put("build", format!("bui build {}", self.entry()));
        if self.typed() {
            put("typecheck", "tsc --noEmit".into());
        }
        put("lint", "biome check .".into());
        put("format", "biome format --write .".into());
        put("fix", "biome check --write .".into());
        let mut doc = json!({
            "name": name,
            "version": "0.1.0",
            "private": true,
            "scripts": scripts,
        });
        if !self.uses_tsx() {
            doc["type"] = json!("module");
        }
        format!("{}\n", serde_json::to_string_pretty(&doc).unwrap_or_default())
    }

    fn tsconfig(&self) -> String {
        let types = if self.pm == PackageManager::Bun { "bun" } else { "node" };
        let mut options = json!({
            "target": "ES2023",
            "module": "ESNext",
            "moduleResolution": "Bundler",
            "strict": true,
            "noUncheckedIndexedAccess": true,
            "verbatimModuleSyntax": true,
            "isolatedModules": true,
            "skipLibCheck": true,
            "noEmit": true,
            "types": [types],
        });
        if self.framework == Framework::React {
            options["jsx"] = json!("react-jsx");
            options["jsxImportSource"] = json!("@blender-ui/react");
        }
        format!("{}\n", serde_json::to_string_pretty(&json!({ "compilerOptions": options, "include": ["src"] })).unwrap_or_default())
    }

    fn packages(&self) -> (Vec<&'static str>, Vec<&'static str>) {
        let mut prod = vec!["@blender-ui/native"];
        match self.framework {
            Framework::React => prod.extend(["@blender-ui/react", "react"]),
            Framework::Svelte => prod.extend(["@blender-ui/svelte", "svelte"]),
            Framework::Vanilla => {}
        }
        let mut dev = vec!["@biomejs/biome"];
        if self.typed() {
            dev.extend(["typescript", if self.pm == PackageManager::Bun { "@types/bun" } else { "@types/node" }]);
            if self.framework == Framework::React {
                dev.push("@types/react");
            }
        }
        if self.uses_tsx() {
            dev.push("tsx");
        }
        (prod, dev)
    }
}

impl Flavour for JsApp {
    fn label(&self) -> String {
        let fw = match self.framework {
            Framework::React => "React",
            Framework::Svelte => "Svelte",
            Framework::Vanilla => "plain DOM",
        };
        let lang = if self.typed() { "TypeScript" } else { "JavaScript" };
        format!("{fw} + {lang} on {}", self.pm.bin())
    }

    fn tools(&self) -> Vec<Tool> {
        let mut tools = match self.pm {
            PackageManager::Bun => vec![Tool::Bun],
            PackageManager::Npm => vec![Tool::Node],
            PackageManager::Pnpm => vec![Tool::Node, Tool::Pnpm],
        };
        if self.framework == Framework::Svelte && !tools.contains(&Tool::Node) {
            tools.push(Tool::Node);
        }
        tools
    }

    fn files(&self, vars: &Vars) -> Vec<File> {
        let flavour = match (self.framework, self.language) {
            (Framework::React, Language::TypeScript) => "js/react-ts",
            (Framework::React, Language::JavaScript) => "js/react-js",
            (Framework::Svelte, _) => "js/svelte",
            (Framework::Vanilla, Language::TypeScript) => "js/vanilla-ts",
            (Framework::Vanilla, Language::JavaScript) => "js/vanilla-js",
        };
        let mut files = render(&["js/common", flavour], vars);
        files.push(File { path: "package.json".into(), body: self.package_json(&vars.name) });
        if self.typed() {
            files.push(File { path: "tsconfig.json".into(), body: self.tsconfig() });
        }
        files
    }

    fn deps(&self, dir: &Path) -> Vec<Step> {
        let (prod, dev) = self.packages();
        vec![
            step("Adding BUI (newest)", self.pm.add(dir, false, &prod)),
            step("Adding TypeScript, Biome and the rest (newest)", self.pm.add(dir, true, &dev)),
            // the generated files in Biome's own style, so `lint` starts clean
            step("Formatting the new files", self.pm.run(dir, "fix")),
        ]
    }

    fn dev(&self, dir: &Path) -> Result<Vec<Cmd>> {
        Ok(vec![self.pm.run(dir, "dev")])
    }

    fn build(&self, dir: &Path) -> Result<Vec<Cmd>> {
        Ok(vec![self.pm.run(dir, "build")])
    }

    fn checks(&self, dir: &Path) -> Vec<Check> {
        let install = Fix::run("installs the packages", vec![Cmd::new(self.pm.bin(), ["install"]).in_dir(dir)]);
        if !dir.join("node_modules").is_dir() {
            return vec![Check::fail("node_modules", "packages are not installed", Some(install))];
        }
        let manifest = read(dir, "node_modules/@blender-ui/native/package.json");
        let have = manifest.and_then(|m| serde_json::from_str::<serde_json::Value>(&m).ok()).and_then(|v| v["version"].as_str().map(str::to_owned));
        let Some(have) = have else {
            return vec![Check::fail("@blender-ui/native", "not installed", Some(install))];
        };
        let fix = Fix::run("updates the packages to the newest", vec![self.pm.update(dir)]);
        vec![check::newest("@blender-ui/native", &have, registry::npm_version("@blender-ui/native").as_deref(), fix)]
    }

    fn update(&self, dir: &Path) -> Vec<Step> {
        vec![step("Updating packages", self.pm.update(dir))]
    }
}
