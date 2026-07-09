precision highp float;
uniform float u_RedShift;
void main() {
    // Every fragment has the same color
    gl_FragColor = vec4(0.5 + u_RedShift, 0.0, 1.0, 1.0);
}