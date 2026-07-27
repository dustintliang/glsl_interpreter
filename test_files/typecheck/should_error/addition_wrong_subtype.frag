precision highp float;
varying Cart3.Point<World> v_Point;
varying Cart3.Direction<World> v_Dir;
void main() {
    Cart3.Point<World> result = v_Point + v_Dir;
    gl_FragColor = vec4(result, 1.0);
}
