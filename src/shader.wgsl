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
    solid: bool,
    color: vec3<f32>,
    refractive_index: f32,
}

struct HitInfo {
    position: vec3<i32>,
    normal: vec3<f32>,
}

fn pcg_hash(input: u32) -> u32 {
    var state = input * 747796405u + 2891336453u;
    var word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

// Refract assuming z axis is perpendicular.
// `r` is new/old.
fn refract_simple(dir: vec3<f32>, r: f32) -> vec3<f32> {
    var d: vec3<f32> = vec3<f32>(dir.x/r, dir.y/r, 0.0);
    let extra: f32 = 1.0 - (d.x * d.x + d.y * d.y);
    d.z = sqrt(extra) * sign(dir.z);
    return select(d, vec3<f32>(dir.x, dir.y, -dir.z), extra < 0.0);
}

// The maximum f32 value
const MAX_F32 = 0x1.fffffep+127f;

// Get the block at a specific coordinate
fn fetch_block(position: vec3<i32>) -> Block {
    let height: i32 = i32(pcg_hash(u32(position.x + position.z * 873))) / 214748364 - 10;
    return Block(height >= position.y, vec3<f32>(1.0, 1.0, 1.0), 1.0);
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
    ), sign((*ray).direction) * hit_axis);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var ray: Ray = Ray(vec3<f32>(0.0, 0.0, 0.0), normalize(vec3<f32>(in.uv.x, in.uv.y, 1.0)));
    var has_hit: bool = false;
    var color: vec3<f32> = vec3<f32>(0.5, 0.5, 0.5);
    for (var i: i32 = 0; i < 100; i++) {
        let hit_info: HitInfo = step(&ray);
        let block: Block = fetch_block(hit_info.position);
        // color = select(color, block.color, block.solid && (!has_hit));
        // Use normal instead
        color = select(color, hit_info.normal * 0.5f + 0.5f, block.solid && (!has_hit));
        has_hit = has_hit || block.solid;
    }
    return vec4<f32>(color, 1.0);
}
