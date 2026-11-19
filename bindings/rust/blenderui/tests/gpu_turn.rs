//! A layer turned on the GPU (the `TURN_SHADER` effect over the box that
//! `turn_box` gives) lands where the CPU turn puts it, with the same pixels.
//! Needs a GPU (Direct3D 11 / OpenGL 3.3); without one the test has nothing
//! to check and passes.

use blenderui::effects::{transform, turn_box, Pixels, TURN_SHADER};
use blenderui::{GpuTarget, Image, ShaderEffect};

/// A quarter turn of a 64 x 32 image (left half red, right half blue) about
/// its centre, on the GPU and on the CPU: the same colours at the same places.
#[test]
fn a_gpu_turn_matches_the_cpu_one() {
    let (w, h) = (64usize, 32usize);
    let rgba: Vec<u8> = (0..w * h)
        .flat_map(|i| if i % w < w / 2 { [255, 0, 0, 255] } else { [0, 0, 255, 255] })
        .collect();
    let origin = (100.0, 100.0);
    let (cx, cy) = (origin.0 + w as f32 / 2.0, origin.1 + h as f32 / 2.0);
    // turn by 90 degrees about the centre: (x, y) -> (cx - (y - cy), cy + (x - cx))
    let m = [0.0, 1.0, -1.0, 0.0, cx + cy, cy - cx];
    let (at, floats) = turn_box(origin, (w as f32, h as f32), &m, 1.0, 1.0).expect("a turnable matrix");
    assert_eq!((at.size.width.round(), at.size.height.round()), (32.0, 64.0), "a quarter turn stands it up");

    let image = Image::from_rgba(w as u32, h as u32, &rgba).expect("an image");
    let (bw, bh) = (at.size.width.round() as usize, at.size.height.round() as usize);
    let Some(mut gpu) = GpuTarget::new(bw, bh) else {
        eprintln!("no GPU device: nothing to compare");
        return;
    };
    let texture = image.texture();
    assert!(texture > 0, "a texture of the layer");
    let turn = ShaderEffect::new("bui-turn", TURN_SHADER, 1, 7).expect("the turn shader");
    let f = [floats[0], floats[1], floats[2], floats[3], floats[4], floats[5], 1.0];
    assert!(gpu.draw(&turn, &[texture], &f), "the shader ran: {}", turn.error());
    let on_gpu = gpu.pixels().to_vec();

    let src = Pixels::from_rgba8(w, h, &rgba);
    let (cpu, cpu_at) = transform(&src, origin, &m).expect("a CPU turn");
    let cpu_rgba = cpu.to_rgba8();
    // the CPU image starts at `cpu_at`, the GPU box at `at`: compare where both have pixels
    let at_gpu = |x: usize, y: usize| on_gpu[y * bw + x] & 0xffffff;
    let at_cpu = |x: usize, y: usize| {
        let (ox, oy) = ((at.origin.x - cpu_at.0).round() as isize, (at.origin.y - cpu_at.1).round() as isize);
        let (cx, cy) = (x as isize + ox, y as isize + oy);
        let k = (cy as usize * cpu.w + cx as usize) * 4;
        u32::from(cpu_rgba[k]) << 16 | u32::from(cpu_rgba[k + 1]) << 8 | u32::from(cpu_rgba[k + 2])
    };
    // well inside each half: the top half of the box came from the left (red) half
    for (x, y) in [(bw / 2, bh / 4), (bw / 2, 3 * bh / 4), (bw / 4, bh / 4), (3 * bw / 4, 3 * bh / 4)] {
        assert_eq!(at_gpu(x, y), at_cpu(x, y), "at ({x}, {y})");
    }
    assert_eq!(at_gpu(bw / 2, bh / 4), 0xff0000, "the left half turned to the top");
    // SAFETY: ours, freed once.
    unsafe { blenderui::sys::BUI_texture_free(texture) };
}
