// Package ui builds BUI's Blender widgets the way MyGo's ui package does: a
// view is a function of the app's state that builds the interface, and runs
// again after input.
//
//	type counter struct{ n int }
//
//	func (s *counter) view(c *ui.Context) {
//		ui.Column(c).Fill().Center().Gap(12).Children(func() {
//			ui.Textf(c, "%d", s.n).FontSize(40).Bold()
//			if ui.PrimaryButton(c, "Increment").Clicked() {
//				s.n++
//			}
//		})
//	}
//
//	bui.App.WhenReady(func() {
//		bui.NewWindow(bui.WindowOptions{Title: "Counter", Content: ui.View(s.view)})
//	})
//	bui.App.Run()
//
// Underneath, every element is a node of the kit, kept from build to build by
// its place among its siblings or its Key: a view that builds the same
// interface again changes nothing, and only what changed is laid out and
// drawn again. Controls take a pointer to what they edit and write the user's
// edit through it as their constructor returns, as in Dear ImGui and egui;
// Clicked, Changed and Submitted report input, and OnClick, OnChange and
// OnSubmit run after the view has built.
//
// Rows and columns lay their children out with flexbox's main ideas (Gap,
// Padding, Grow, Justify, AlignItems, Width and Height in DIPs), on the kit's
// packer. The look is Blender's.
//
// SPDX-License-Identifier: GPL-2.0-or-later
package ui

import (
	"fmt"
	"reflect"

	"github.com/ppmpreetham/BUI/bindings/go/bui"
)

// Context is a window's build state; the view gets the same one every time.
// Its current parent changes inside Children callbacks.
type Context struct {
	view     func(*Context)
	win      *bui.Window
	ctx      *bui.Context
	root     *slot
	parent   *slot
	key      any
	keyed    bool
	pass     int
	actions  []func()
	consumed bool // a control took input in this pass
	dirty    bool // input arrived since the last build
	building bool
	scale    float32 // pixels per DIP
	keys     map[string]*shortcut
	theme    Theme
	themeGen int
}

type shortcut struct{ pending int }

// Content is a window's user interface, for bui.WindowOptions.Content. Make it
// with View.
type Content struct{ view func(*Context) }

// View returns the content of a window whose interface view builds:
//
//	bui.NewWindow(bui.WindowOptions{Title: "Counter", Content: ui.View(app.view)})
//
// view runs on the main thread whenever the window needs a frame after
// input, Context.Invalidate or Window.Invalidate. A Content can serve several
// windows, each with its own element state.
func View(view func(c *Context)) *Content { return &Content{view: view} }

// AttachContent builds the view into the window's tree (bui.NewWindow calls it).
func (v *Content) AttachContent(w *bui.Window) {
	c := newContext(v.view, w.Context(), w)
	w.OnFrame(func() {
		if c.dirty {
			c.build()
		}
	})
	c.build()
}

func newContext(view func(*Context), ctx *bui.Context, win *bui.Window) *Context {
	c := &Context{view: view, win: win, ctx: ctx}
	root := ctx.Root()
	c.root = &slot{kind: kindColumn, node: root, host: root, index: map[ident]*slot{}}
	return c
}

// build runs the view until its input is consumed: a pass whose controls took
// input, or whose actions ran, is built once more, so the frame shows their
// outcome.
func (c *Context) build() {
	c.building = true
	defer func() { c.building = false }()
	c.dirty = false
	c.scale = float32(bui.Unit()) / 20
	if g := bui.ThemeGeneration(); g != c.themeGen || c.pass == 0 {
		c.themeGen = g
		c.theme = currentTheme()
	}
	for i := 0; i < 4; i++ {
		c.pass++
		c.consumed = false
		c.parent = c.root
		c.begin(c.root)
		c.view(c)
		c.keyed, c.key = false, nil
		c.finish(c.root)
		actions := c.actions
		c.actions = c.actions[:0]
		for _, fn := range actions {
			fn()
		}
		if !c.consumed && len(actions) == 0 {
			break
		}
	}
}

// markDirty asks for a build after input.
func (c *Context) markDirty() {
	c.dirty = true
	if c.win != nil && !c.building {
		c.win.Invalidate()
	}
}

// Key gives the next element a key, so its state follows the item it shows
// when the collection around it changes. Keys are unique among siblings and
// compare as map keys do; a key Go cannot compare compares by its value
// printed.
func (c *Context) Key(key any) *Context {
	c.key, c.keyed = key, true
	return c
}

// Invalidate asks for another build and frame.
func (c *Context) Invalidate() {
	if c.building {
		c.consumed = true
		return
	}
	c.markDirty()
}

// Theme is the colors the window's widgets use.
func (c *Context) Theme() Theme { return c.theme }

// Window is the window the view builds, nil in a Tester.
func (c *Context) Window() *bui.Window { return c.win }

// Shortcut reports that a key was pressed since the last build: mods (Shift,
// Ctrl, Alt, Cmd) and a key, as c.Shortcut(Ctrl, KeyS).
func (c *Context) Shortcut(mods Modifiers, key Key) bool {
	combo := comboName(mods, key)
	sc := c.keys[combo]
	if sc == nil {
		if c.keys == nil {
			c.keys = map[string]*shortcut{}
		}
		sc = &shortcut{}
		c.keys[combo] = sc
		c.ctx.Bind(combo, func() {
			sc.pending++
			c.markDirty()
		})
	}
	if sc.pending == 0 {
		return false
	}
	sc.pending--
	c.consumed = true
	return true
}

// -- identity ----------------------------------------------------------------------

type ident struct {
	kind kind
	n    int // place among the unkeyed siblings of its kind, -1 when keyed
	key  any
}

func keyOf(k any) any {
	if k == nil {
		return nil
	}
	if t := reflect.TypeOf(k); !t.Comparable() {
		return fmt.Sprint(k)
	}
	if f, ok := k.(float64); ok && f != f { // NaN equals nothing
		return fmt.Sprint(k)
	}
	return k
}

// child returns the slot of the next element of a kind in the current
// parent: the one it was last time, or a new one.
func (c *Context) child(k kind, typ string) *slot {
	p := c.parent
	var id ident
	if c.keyed {
		id = ident{kind: k, n: -1, key: keyOf(c.key)}
		c.keyed, c.key = false, nil
	} else {
		id = ident{kind: k, n: p.ord[k]}
		p.ord[k]++
	}
	s := p.index[id]
	if s != nil && s.pass == c.pass { // a key used twice: the second gets one of its own
		id.n = -2 - len(p.kids)
		s = p.index[id]
	}
	if s == nil {
		s = c.newSlot(k, typ, p)
		p.index[id] = s
	}
	s.pass = c.pass
	s.look = look{}
	s.lay = layout{}
	p.kids = append(p.kids, s)
	return s
}

func (c *Context) newSlot(k kind, typ string, p *slot) *slot {
	s := &slot{kind: k, parent: p, node: bui.NewNode(typ)}
	switch k {
	case kindColumn, kindRow, kindRadioGroup:
		s.host = s.node
	case kindScroll:
		s.host = bui.NewNode("frame")
		s.node.Add(s.host)
	case kindCollapsible:
		// a panel's other children are header widgets: the body is its first
		s.host = bui.NewNode("frame")
		s.node.Child(0).Add(s.host)
	}
	if s.host != nil {
		s.index = map[ident]*slot{}
	}
	s.node.OnClick(func(bui.Event) {
		s.clicks++
		c.markDirty()
	})
	s.node.OnChange(func(bui.Event) { c.markDirty() })
	s.node.OnSubmit(func(bui.Event) {
		s.submits++
		c.markDirty()
	})
	p.host.Add(s.node)
	return s
}

// begin starts a container's children for this pass.
func (c *Context) begin(s *slot) {
	s.begun = c.pass
	s.kids = s.kids[:0]
	s.ord = [kindCount]int{}
}

// Children builds the element's children: what fn builds goes inside it. It
// does nothing for elements that hold no children (controls).
func (e Element) Children(fn func()) Element {
	s := e.s
	if s == nil || s.host == nil {
		return e
	}
	c := e.c
	prev := c.parent
	c.begin(s)
	c.parent = s
	fn()
	c.parent = prev
	return e
}
