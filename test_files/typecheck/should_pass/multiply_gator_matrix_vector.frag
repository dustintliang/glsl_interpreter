precision highp float;
uniform Hom4.Matrix<Object, World> u_Model;
varying Hom4.vector<Object> v_Position;
void main() {
    Hom4.vector<World> result = u_Model * v_Position;
    gl_FragColor = result;
}
