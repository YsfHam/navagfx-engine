// Vertex shader


struct CameraUniform {
    view_proj: mat4x4<f32>
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct QuadInstanceDataInput {
    @location(2) model_matrix_1: vec4<f32>,
    @location(1) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_2: vec4<f32>,
    @location(4) model_matrix_3: vec4<f32>,

    @location(5) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: QuadInstanceDataInput
) -> VertexOutput {

    let model_matrix = mat4x4<f32> (
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3
    );

    var out: VertexOutput;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 0.0, 1.0);

    out.color = instance.color;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
