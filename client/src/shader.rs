use macroquad::{math::Vec2, miniquad::UniformDesc, prelude::*, ui::{self}};

const FRAGMENT_SHADER: &str = include_str!("starfield.glsl");

const VERTEX_SHADER: &str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying float iTime;

uniform mat4 Model;
uniform mat4 Projection;
uniform vec4 _Time;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    iTime = _Time.x;
}
";

pub(crate) fn load_starfield_shader() -> Material {
    load_material(
        VERTEX_SHADER,
        FRAGMENT_SHADER,
        MaterialParams {
            uniforms: vec![
                ("iResolution".to_string(), UniformType::Float2),
                ("global_position".to_string(), UniformType::Float2),
            ],
            ..Default::default()
        },
    )
    .unwrap()
}

/// Applies a shader to a render target and draws it to the screen.
/// 
/// # Arguments
/// 
/// * `render_target` - The render target containing the texture to draw
/// * `shader` - The shader to apply to the texture
/// * `global_position` - A value that can modify shader behavior (e.g., for direction-based effects)
pub fn apply_shader_to_screen(render_target: RenderTarget, shader: Material, global_position: Vec2) {
    shader.set_uniform("iResolution", (screen_width(), screen_height()));
    shader.set_uniform("global_position", global_position.to_array());
    
    gl_use_material(shader);
    
    draw_texture_ex(
        render_target.texture,
        0.,
        0.,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(screen_width(), screen_height())),
            ..Default::default()
        },
    );
    
    gl_use_default_material();
}