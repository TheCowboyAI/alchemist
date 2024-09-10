precision mediump float;

uniform float u_time;
uniform vec2 u_resolution;

uniform vec2 L;
uniform float M;

// https://iquilezles.org/articles/distfunctions2d/
float sdfCircle(vec2 p, float r) {
  // note: sqrt(pow(p.x, 2.0) + pow(p.y, 2.0)) - r;
  return length(p) - r;
}

void main() {
  // note: set up uv coordinates
  vec2 uv = gl_FragCoord.xy / u_resolution;
  vec2 c = ;
  vec2 k = ;

// gl_FragColor = vec4(color, 1.0);
}