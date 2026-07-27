precision highp float;
varying Cart3.Point<World> v_A;
varying Cart3.Point<Model> v_B;
void main() {
    Cart3.Point<World> result = v_A + v_B;
    gl_FragColor = vec4(result, 1.0);
}
