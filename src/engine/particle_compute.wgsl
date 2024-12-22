struct Particle
{
    old_position : vec2<f32>,
    position : vec2<f32>,
}

@group(0) @binding(0)
var<storage, read_write> particles : array<Particle>;

struct Uniforms
{
    side_length : u32,
    mouse : vec2<f32>,
}

@group(0) @binding(1)
var<uniform> uniforms : Uniforms;

fn physics(index : u32)
{
    var velocity : vec2<f32> = particles[index].position - particles[index].old_position;

    // F = (G * m1 * m2) / d^2
    let center : vec2<f32> = vec2<f32>(f32(uniforms.side_length) / 2., f32(uniforms.side_length) / 2.);
    // let center : vec2<f32> = uniforms.mouse;
    let dist : f32 = distance(center, particles[index].position);
    let force : f32= (9. * 1. * 100.) / (dist);
    let acc : f32 = force / 0.1;
    let impulse : vec2<f32> = force * (1. / 60.) * normalize(center - particles[index].position);
     velocity += impulse;


    if particles[index].position.y < 0.5
    {
        velocity = velocity * -1.;
    }

    particles[index].old_position = particles[index].position;
    particles[index].position += velocity;
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>)
{
    physics(global_id.x + (global_id.y * uniforms.side_length));
}