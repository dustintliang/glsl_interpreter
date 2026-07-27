precision highp float;
varying Cart3.Point<World> v_Position;
varying Color3<RGB> v_Color;
void main() {
    Cart3.Point<World> result = v_Position + v_Color;
    gl_FragColor = vec4(result, 1.0);
}
