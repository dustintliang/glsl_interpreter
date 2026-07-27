precision highp float;
uniform mat4 u_Model;
varying vec4 v_Position;
void main() {
    vec4 result = u_Model * v_Position;
    gl_FragColor = result;
}
