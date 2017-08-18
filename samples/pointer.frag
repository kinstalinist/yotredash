#version 140

out vec4 color;

uniform vec4 pointer;

void main() {
    color = mix(vec4(1.0, 0.0, 0.0, 1.0), vec4(0.0, 1.0, 0.0, 1.0), distance(gl_FragCoord.xy, pointer.xy) / 100.0);
    color.rgb *= (pointer.z != 0.0) ? -1.0 : 1.0;
}
