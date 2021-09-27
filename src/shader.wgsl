// Vertex shader

struct VertexInput {
    [[location(0)]] position: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(1)]]position: vec2<f32>;
};

[[stage(vertex)]]
fn main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = model.position;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    return out;
}

// Fragment shader

// wgsl cant seem to do compile time math
// @TODO feed some of those as uniforms at some point
let GFX_WIDTH_DEFAULT: u32 = 64u;
let GFX_HEIGHT_DEFAULT: u32 = 32u;

let STORAGE_BITS: u32 = 32u; // sizeof(u32) * 8;
let PACKED_WIDTH: u32  = 2u; // (GFX_WIDTH_DEFAULT / STORAGE_BITS)

[[block]]
struct GfxState {
    pixels: array<u32, 64>; // (PACKED_WIDTH * GFX_HEIGHT_DEFAULT)
};
[[group(0), binding(0)]]
var<uniform> gfx_state: GfxState;

struct DataPos {
    col: u32;
    nibble: u32;
};

fn get_bucket(x: u32, y: u32) -> DataPos {
    var real_x = u32(x / STORAGE_BITS);
    var pos: DataPos;
    pos.col = (y * PACKED_WIDTH) + real_x;
    pos.nibble = (STORAGE_BITS * (real_x + 1u)) - x - 1u;
    return pos;
}

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var st = (in.position + vec2<f32>(1.0, 1.0)) * 0.5;
    var color = vec3<f32>(st.x, 0.0, st.y); // colorize!

    var width = 64.0;
    var height = 32.0;

    var x: u32 = u32(floor(width * st.x));
    var y: u32 = u32(floor(height * (1.0 - st.y)));

    var bucket: DataPos = get_bucket(x, y);
    var mask = 1u << bucket.nibble;
    var col = bucket.col;

    color = color * f32(clamp(gfx_state.pixels[col] & mask, 0u, 1u));

    return vec4<f32>(color, 1.0);
}
