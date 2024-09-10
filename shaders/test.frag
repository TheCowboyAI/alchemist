#ifdef GL_ES
precision mediump float;
#endif

vec4 color = vec4(0.2,0.3,0.1,1.0);

void main(){
  gl_FragColor = color;
}