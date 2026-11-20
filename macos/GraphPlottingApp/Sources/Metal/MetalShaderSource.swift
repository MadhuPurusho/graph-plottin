// MetalShaderSource.swift
// Runtime-compiled Metal Shading Language source for 120 FPS hardware-accelerated graph rendering.

import Foundation

enum MetalShaderSource {
    static let source: String = """
#include <metal_stdlib>
using namespace metal;

// ─────────────────────────────────────────────────────────────────────────────
// Uniforms & Structures
// ─────────────────────────────────────────────────────────────────────────────

struct GraphUniforms {
    float2 viewport_min; // (x_min, y_min) in world space
    float2 viewport_max; // (x_max, y_max) in world space
    float2 screen_size;  // (width, height) in screen points/pixels
    float2 _padding;
    float4 axis_color;
    float4 grid_color;
    float4 subgrid_color;
};

struct VertexOut {
    float4 position [[position]];
    float2 world_pos;
    float2 screen_pos;
};

struct CurveVertexOut {
    float4 position [[position]];
    float2 world_pos;
};

struct PointVertexOut {
    float4 position [[position]];
    float2 local_pos; // [-1, 1] across quad
    float4 color;
};

// ─────────────────────────────────────────────────────────────────────────────
// Grid & Axis Shaders (Fullscreen Quad)
// ─────────────────────────────────────────────────────────────────────────────

vertex VertexOut grid_vertex(uint vertex_id [[vertex_id]],
                             constant GraphUniforms &uniforms [[buffer(0)]]) {
    float2 positions[4] = {
        float2(-1.0, -1.0),
        float2( 1.0, -1.0),
        float2(-1.0,  1.0),
        float2( 1.0,  1.0)
    };

    float2 p = positions[vertex_id];
    VertexOut out;
    out.position = float4(p, 0.0, 1.0);

    out.screen_pos = float2((p.x + 1.0) * 0.5 * uniforms.screen_size.x,
                            (1.0 - p.y) * 0.5 * uniforms.screen_size.y);

    out.world_pos = float2(
        mix(uniforms.viewport_min.x, uniforms.viewport_max.x, (p.x + 1.0) * 0.5),
        mix(uniforms.viewport_min.y, uniforms.viewport_max.y, (p.y + 1.0) * 0.5)
    );

    return out;
}

fragment float4 grid_fragment(VertexOut in [[stage_in]],
                              constant GraphUniforms &uniforms [[buffer(0)]]) {
    float2 v_span = uniforms.viewport_max - uniforms.viewport_min;
    if (v_span.x <= 0.0 || v_span.y <= 0.0) {
        return float4(0.98, 0.98, 0.99, 1.0);
    }

    float2 world_per_px = v_span / uniforms.screen_size;
    float4 bg = float4(0.98, 0.98, 0.99, 1.0);

    // Dynamic 1-2-5 grid spacing calculation
    float target_screen_spacing = 80.0;
    float target_world_step = target_screen_spacing * world_per_px.x;
    float exponent = floor(log10(target_world_step));
    float base_step = pow(10.0, exponent);
    float fraction = target_world_step / base_step;

    float major_step = base_step;
    if (fraction > 4.5) {
        major_step = base_step * 5.0;
    } else if (fraction > 1.8) {
        major_step = base_step * 2.0;
    }

    float minor_step = major_step * 0.2;

    float2 dist_minor = abs(fract((in.world_pos / minor_step) + 0.5) - 0.5) * minor_step;
    float2 dist_major = abs(fract((in.world_pos / major_step) + 0.5) - 0.5) * major_step;

    float2 px_minor = dist_minor / world_per_px;
    float2 px_major = dist_major / world_per_px;

    float line_minor_x = smoothstep(1.0, 0.0, px_minor.x);
    float line_minor_y = smoothstep(1.0, 0.0, px_minor.y);
    float minor_alpha = max(line_minor_x, line_minor_y) * 0.25;

    float line_major_x = smoothstep(1.2, 0.0, px_major.x);
    float line_major_y = smoothstep(1.2, 0.0, px_major.y);
    float major_alpha = max(line_major_x, line_major_y) * 0.55;

    float2 dist_axis = abs(in.world_pos);
    float2 px_axis = dist_axis / world_per_px;
    float axis_x = smoothstep(1.5, 0.0, px_axis.x);
    float axis_y = smoothstep(1.5, 0.0, px_axis.y);
    float axis_alpha = max(axis_x, axis_y) * 0.95;

    float4 color = bg;
    color = mix(color, uniforms.subgrid_color, minor_alpha);
    color = mix(color, uniforms.grid_color, major_alpha);
    color = mix(color, uniforms.axis_color, axis_alpha);

    return color;
}

// ─────────────────────────────────────────────────────────────────────────────
// Curve Stroke Shaders
// ─────────────────────────────────────────────────────────────────────────────

vertex CurveVertexOut curve_vertex(uint vertex_id [[vertex_id]],
                                   constant float2 *points [[buffer(0)]],
                                   constant GraphUniforms &uniforms [[buffer(1)]]) {
    float2 p_world = points[vertex_id];
    float2 v_span = uniforms.viewport_max - uniforms.viewport_min;

    float norm_x = (p_world.x - uniforms.viewport_min.x) / v_span.x * 2.0 - 1.0;
    float norm_y = (p_world.y - uniforms.viewport_min.y) / v_span.y * 2.0 - 1.0;

    CurveVertexOut out;
    out.position = float4(norm_x, norm_y, 0.0, 1.0);
    out.world_pos = p_world;
    return out;
}

fragment float4 curve_fragment(CurveVertexOut in [[stage_in]],
                               constant float4 &curve_color [[buffer(0)]]) {
    return curve_color;
}

// ─────────────────────────────────────────────────────────────────────────────
// Shaded Area (Confidence Band / Integral) Shaders
// ─────────────────────────────────────────────────────────────────────────────

vertex CurveVertexOut area_vertex(uint vertex_id [[vertex_id]],
                                  constant float2 *points [[buffer(0)]],
                                  constant GraphUniforms &uniforms [[buffer(1)]]) {
    float2 p_world = points[vertex_id];
    float2 v_span = uniforms.viewport_max - uniforms.viewport_min;

    float norm_x = (p_world.x - uniforms.viewport_min.x) / v_span.x * 2.0 - 1.0;
    float norm_y = (p_world.y - uniforms.viewport_min.y) / v_span.y * 2.0 - 1.0;

    CurveVertexOut out;
    out.position = float4(norm_x, norm_y, 0.0, 1.0);
    out.world_pos = p_world;
    return out;
}

fragment float4 area_fragment(CurveVertexOut in [[stage_in]],
                              constant float4 &area_color [[buffer(0)]]) {
    return area_color;
}

// ─────────────────────────────────────────────────────────────────────────────
// Point Marker Shaders (Instanced Billboard Circles)
// ─────────────────────────────────────────────────────────────────────────────

vertex PointVertexOut point_vertex(uint vertex_id [[vertex_id]],
                                   uint instance_id [[instance_id]],
                                   constant float2 *positions [[buffer(0)]],
                                   constant GraphUniforms &uniforms [[buffer(1)]],
                                   constant float4 &point_color [[buffer(2)]],
                                   constant float &point_radius_px [[buffer(3)]]) {
    float2 quad_offsets[4] = {
        float2(-1.0, -1.0),
        float2( 1.0, -1.0),
        float2(-1.0,  1.0),
        float2( 1.0,  1.0)
    };

    float2 p_world = positions[instance_id];
    float2 v_span = uniforms.viewport_max - uniforms.viewport_min;

    float center_x = (p_world.x - uniforms.viewport_min.x) / v_span.x * 2.0 - 1.0;
    float center_y = (p_world.y - uniforms.viewport_min.y) / v_span.y * 2.0 - 1.0;

    float2 ndc_radius = (float2(point_radius_px, point_radius_px) / uniforms.screen_size) * 2.0;

    float2 offset = quad_offsets[vertex_id];
    float2 ndc_pos = float2(center_x, center_y) + offset * ndc_radius;

    PointVertexOut out;
    out.position = float4(ndc_pos, 0.0, 1.0);
    out.local_pos = offset;
    out.color = point_color;
    return out;
}

fragment float4 point_fragment(PointVertexOut in [[stage_in]]) {
    float dist = length(in.local_pos);
    if (dist > 1.0) {
        discard_fragment();
    }

    float alpha = smoothstep(1.0, 0.85, dist);
    float border_width = 0.28;
    float is_border = smoothstep(1.0 - border_width - 0.05, 1.0 - border_width, dist);

    float4 fill_color = in.color;
    float4 border_color = float4(1.0, 1.0, 1.0, 1.0);

    float4 final_color = mix(fill_color, border_color, is_border);
    final_color.a *= alpha;

    return final_color;
}
"""
}
