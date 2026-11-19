// Ground mist: scattering that thins exponentially with height (the form vork admits for world fog).
import vork;
struct Inputs {
    float density = 0.03;     // scattering per metre at base_height
    float falloff = 0.25;     // per metre
    float base_height = 0.0;
    float anisotropy = 0.35;
};
MediumCoefficients medium(MediumContext ctx, Inputs inputs) {
    MediumCoefficients m;
    float d = inputs.density * exp(-inputs.falloff * (ctx.world_position.y - inputs.base_height));
    m.scattering = float3(d, d, d);
    m.anisotropy = inputs.anisotropy;
    return m;
}
