struct Uniforms {
    time: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var world_tex: texture_3d<u32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.uv = model.position.xy;
    return out;
}


struct Ray {
    position: vec3<f32>,
    direction: vec3<f32>,
}

struct Block {
    opacity: f32,
    color: vec3<f32>,
    refractive: bool,
}

struct HitInfo {
    position: vec3<i32>,
    normal: vec3<f32>,
    uv: vec2<f32>,
}

fn pcg_hash(input: u32) -> u32 {
    var state = input * 747796405u + 2891336453u;
    var word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

/*
// Refract assuming z axis is perpendicular.
// `r` is new/old.
fn refract_simple(dir: vec3<f32>, r: f32) -> vec3<f32> {
    var d: vec3<f32> = vec3<f32>(dir.x/r, dir.y/r, 0.0);
    let extra: f32 = 1.0 - (d.x * d.x + d.y * d.y);
    d.z = sqrt(extra) * sign(dir.z);
    return select(d, vec3<f32>(dir.x, dir.y, -dir.z), extra < 0.0);
}
*/

// The maximum f32 value
const MAX_F32 = 0x1.fffffep+127f;

fn rand(pos: vec3<i32>) -> u32 {
    return pcg_hash(u32(pos.x + pos.y * 873 + pos.z * 93857));
}

/*
// Get the block at a specific coordinate
fn fetch_block(position: vec3<i32>) -> Block {
    let height: i32 = rand(position.xz);
    return Block(min(height, -2) >= position.y, vec3<f32>(1.0, 1.0, 1.0), select(1.0, 1.3, height > -2 && position.y <= height));
}
*/

fn fetch_block(position: vec3<i32>) -> Block {
    let world_size: vec3<u32> = textureDimensions(world_tex);
    let texel: vec3<u32> = vec3<u32>(position + vec3<i32>(world_size/2));
    let data: u32 = textureLoad(world_tex, texel, 0).r;
    /*
    4 bits per channel, but alpha uses the first bit for
    refractivity, and the last 3 bits for opacity
    */
    let opacity: f32 = f32(data & 7) / 7.0;
    let refractive: bool = ((data >> 3
    ) & 1) > 0;
    let rng: u32 = rand(position);
    let color: vec3<f32> = vec3<f32>(
        f32((data >> 12) & 15) / 16.0 + f32((rng >> 0) & 1) / 32.0,
        f32((data >> 8) & 15) / 16.0 + f32((rng >> 1) & 1) / 32.0,
        f32((data >> 4) & 15) / 16.0 + f32((rng >> 2) & 1) / 32.0
    );
    return Block(select(
        -1.0, opacity,
        (texel.x < world_size.x) &&
        (texel.y < world_size.y) &&
        (texel.z < world_size.z)
    ), color, refractive);
}

// Simulate a ray taking one step
// Doesn't include refraction or reflection,
// just traversal of the voxel grid.
fn step(ray: ptr<function, Ray>) -> HitInfo {
    // select(false_value, true_value, condition)
    let next_coordinate: vec3<f32> = vec3<f32>(
        select(
            select(floor((*ray).position.x), ceil((*ray).position.x), (*ray).direction.x >= 0.0),
            floor((*ray).position.x) + sign((*ray).direction.x),
            trunc((*ray).position.x) == (*ray).position.x
        ),
        select(
            select(floor((*ray).position.y), ceil((*ray).position.y), (*ray).direction.y >= 0.0),
            floor((*ray).position.y) + sign((*ray).direction.y),
            trunc((*ray).position.y) == (*ray).position.y
        ),
        select(
            select(floor((*ray).position.z), ceil((*ray).position.z), (*ray).direction.z >= 0.0),
            floor((*ray).position.z) + sign((*ray).direction.z),
            trunc((*ray).position.z) == (*ray).position.z
        ),
    );
    var distance: vec3<f32> = vec3<f32>(
        select(MAX_F32, (next_coordinate.x - (*ray).position.x) / (*ray).direction.x, (*ray).direction.x != 0.0),
        select(MAX_F32, (next_coordinate.y - (*ray).position.y) / (*ray).direction.y, (*ray).direction.y != 0.0),
        select(MAX_F32, (next_coordinate.z - (*ray).position.z) / (*ray).direction.z, (*ray).direction.z != 0.0),
    );
    let hit_axis: vec3<f32> = vec3<f32>(
        select(0.0f, 1.0f, distance.x <= distance.y && distance.x <= distance.z),
        select(0.0f, 1.0f, distance.y <  distance.x && distance.y <= distance.z),
        select(0.0f, 1.0f, distance.z <  distance.x && distance.z <  distance.y),
    );
    let non_hit_axis: vec3<f32> = vec3<f32>(1.0f, 1.0f, 1.0f) - hit_axis;
    (*ray).position += (*ray).direction * min(distance.x, min(distance.y, distance.z));
    // Make the hit axis more precise
    (*ray).position = (*ray).position * non_hit_axis + round((*ray).position) * hit_axis;
    return HitInfo(vec3<i32>(
        non_hit_axis * floor((*ray).position) +
        hit_axis * (*ray).position -
        f32(dot(hit_axis, (*ray).direction) < 0.0) * hit_axis
    ), -1.0 * sign((*ray).direction) * hit_axis,
    select(select((*ray).position.xz, (*ray).position.yz, hit_axis.x == 1.0), (*ray).position.xy, hit_axis.z == 1.0)
    );
}

fn border(uv: vec2<f32>) -> bool {
    return uv.x < 0.1 || uv.x > 0.9 || uv.y < 0.1 || uv.y > 0.9;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let world_size: vec3<u32> = textureDimensions(world_tex);
    var ray: Ray = Ray(vec3<f32>(0.0, 0.0, uniforms.time), normalize(vec3<f32>(in.uv.x, in.uv.y, 1.0)));
    var color_left: f32 = 1.0;
    var color: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
    var old_refract: f32 = 1.0;
    for (var i: i32 = 0; i < 64; i++) {
        let hit_info: HitInfo = step(&ray);
        let block: Block = fetch_block(hit_info.position);
        if (block.opacity == -1.0) {
            break;
        }
        let refractive_index: f32 = select(1.0, 1.5, block.refractive);
        let refracted: vec3<f32> = refract(ray.direction, hit_info.normal, old_refract/refractive_index);
        old_refract = refractive_index;
        ray.direction = select(refracted, reflect(ray.direction, hit_info.normal), refracted == vec3<f32>(0.0, 0.0, 0.0));
        // ray.direction = select(ray.direction, reflect(ray.direction, hit_info.normal), block.refractive_index > 1.0);
        // color = select(color, block.color, block.solid && (!has_hit));
        // Use normal instead
        color += block.opacity * block.color * color_left;
        color_left = color_left * (1.0 - block.opacity);
    }
    // We have escaped the view distance.
    // Let's draw the sky now.
    color += color_left * vec3<f32>(0.2, 0.2, 0.9 + 0.2 * ray.direction.y);
    return vec4<f32>(color, 1.0);
}
