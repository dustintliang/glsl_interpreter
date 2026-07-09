precision highp float;

uniform Hom4.Matrix<Object, Model> u_Model;
uniform Hom4.Matrix<Model, World> u_World;
uniform Hom4.InvTrMatrix<Object, World> u_ModelWorldInverseTranspose;
uniform Hom4.Matrix<World, Camera> u_Camera;

uniform Cart3.Point<World> u_Light;
uniform float u_SpecPower;

varying Cart3.Point<Model> v_Position;
varying Cart3.Direction<World> v_Normal;

void main() {
    // Calculate our world position
    Cart3.Point<World> worldPosition = vec3(u_World * u_Model * vec4(v_Position, 1.0));

    // Calculate our world space normal
    Cart3.Direction<World> worldNormal = normalize(vec3(u_ModelWorldInverseTranspose * vec4(v_Normal, 0.0)));

    // Our light direction is just our normalized light source
    Cart3.Direction<World> lightDir = normalize(u_Light - worldPosition);

    // Calculate our diffuse amount
    float diffuse = max(dot(lightDir, worldNormal), 0.0);

    // Calculate our reflection across the normal
    // see https://learnopengl.com/Lighting/Basic-Lighting for more details
    Cart3.Direction<World> reflectDir = normalize(reflect(-lightDir, worldNormal)); // reflect the light past our normal

    // Convert our reflection direction into camera space
    // Note that we do not need the inverse transpose (it's not a normal anymore)
    // But it's still a direction, so throw out any translation that would happen
    Cart3.Direction<Camera> cameraReflectDir = vec3(u_Camera * vec4(reflectDir, 0.0));

    // next, calculate position in camera space
    Cart3.Point<Camera> cameraSpacePosition = vec3(u_Camera * vec4(worldPosition, 1.0));

    // our camera is at the origin of camera space, so calculate direction from that
    Cart3.Point<Camera> cameraDir = normalize(vec3(0.0, 0.0, 0.0) - cameraSpacePosition);

    // calculate the angle between the cameraDir and
    //   the reflected light direction _toward_ the camera(in camera space)
    float angle = max(dot(cameraDir, cameraReflectDir), 0.0);
    // calculate fall-off with power
    float specular = max(pow(angle, u_SpecPower), 0.0);

    // set constant colors for this demo
    Color3<RGB> diffuseColor = vec3(1.0, 0.3, 0.7);
    Color3<RGB> specularColor = vec3(1.0, 1.0, 1.0);
    Color3<RGB> ambientColor = vec3(0.0, 0.0, 0.0);

    // add up our components
    Color3<RGB> color = ambientColor + diffuse * diffuseColor + specular * specularColor;

    gl_FragColor = vec4(color, 1.0);
}
