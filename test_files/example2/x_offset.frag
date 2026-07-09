precision highp float; // cruft
varying Color3<RGB> v_Color;
varying Cart3.Point<Clip> v_Position;
void main() {
    // Use our rasterized v_Color;
    Color3<RGB> color = v_Color;
    color.g += v_Position.x; // Add our y-value to the red component
    gl_FragColor = vec4(color, 1.0);
}
