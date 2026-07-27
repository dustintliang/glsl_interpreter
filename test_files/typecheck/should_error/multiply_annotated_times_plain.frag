precision highp float;
uniform Hom4.Matrix<Object, Model> u_Model;
varying vec4 v_Position;
void main() {
    Hom4.vector<Model> result = u_Model * v_Position;
    gl_FragColor = result;
}
