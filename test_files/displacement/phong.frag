precision highp float;
varying Cart3.Point<Object> v_Position;
varying Cart3.Direction<Object> v_Normal;

uniform Hom4.Matrix<Object, Model> u_Model;
uniform Hom4.Matrix<Model, World> u_World;
uniform Hom4.Matrix<World, Camera> u_Camera;
uniform Hom4.InvTrMatrix<Object, World> u_ModelWorldInverseTranspose;
uniform Cart3.Direction<World> u_Light;

void main() {
    // set constant colors and specular power for this demo
    Color3<RGB> diffuseColor = vec3(0.7, 0.7, 0.8);
    Color3<RGB> specularColor = vec3(1.0, 1.0, 1.0);
    Color3<RGB> ambientColor = vec3(0.0, 0.0, 0.0);
    float specPower = 16.0;

    // Calculate our world position and normals
    Cart3.Point<World> worldPosition = vec3(u_World * u_Model * vec4(v_Position, 1.0));
    Cart3.Direction<World> worldNormal = normalize(vec3(u_ModelWorldInverseTranspose * vec4(v_Normal, 0.0)));

    // Calculate diffuse "amount"
    Cart3.Direction<World> lightDir = normalize(u_Light);
    float diffuse = max(dot(lightDir, worldNormal), 0.0);

    // Calculate our reflection across the normal and convert it into camera space
    // see https://learnopengl.com/Lighting/Basic-Lighting for more details
    Cart3.Direction<World> reflectDir = normalize(reflect(-lightDir, worldNormal));
    Cart3.Direction<Camera> cameraReflectDir = vec3(u_Camera * vec4(reflectDir, 0.0));

    // Calculate our camera position and direction
    Cart3.Point<Camera> cameraSpacePosition = vec3(u_Camera * vec4(worldPosition, 1.0));
    Cart3.Direction<Camera> cameraDir = normalize(vec3(0.0, 0.0, 0.0) - cameraSpacePosition);

    // Calculate specular based on the reflection into the camera
    float angle = max(dot(cameraDir, cameraReflectDir), 0.0);
    float specular = max(pow(angle, specPower), 0.0);

    // add up our components
    Color3<RGB> color = ambientColor +
        diffuse * diffuseColor +
        specular * specularColor;

    gl_FragColor = vec4(color, 1.0);
}
