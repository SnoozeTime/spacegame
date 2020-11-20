// THANK YOU https://www.shadertoy.com/view/XltGR2

// TODO https://www.shadertoy.com/view/Mlcczr
in vec2 v_uv;
in vec4 v_color;
out vec4 frag;

uniform float u_time;

const float PI = 3.14159265;



vec2 hash2( vec2 p )
{
    // texture based white noise
    //	return texture( iChannel0, (p+0.5)/256.0, -100.0 ).xy;

    // procedural white noise
    return fract(sin(vec2(dot(p,vec2(127.1,311.7)),dot(p,vec2(269.5,183.3))))*43758.5453);
}

vec3 voronoi( in vec2 x )
{
    vec2 n = floor(x);
    vec2 f = fract(x);
//
//    //----------------------------------
//    // first pass: regular voronoi
//    //----------------------------------
    vec2 mg, mr;
//
    float md = 8.0;
    for( int j=-1; j<=1; j++ )
    for( int i=-1; i<=1; i++ )
    {
        vec2 g = vec2(float(i),float(j));
        vec2 o = hash2( n + g );
        o = 0.5 + 0.5*sin( u_time + 6.2831*o );
        vec2 r = g + o - f;
        float d = dot(r,r);

        if( d<md )
        {
            md = d;
            mr = r;
            mg = g;
        }
    }

    //----------------------------------
    // second pass: distance to borders
    //----------------------------------
//
//    md = 8.0;
//    for( int j=-2; j<=2; j++ )
//    for( int i=-2; i<=2; i++ )
//    {
//        vec2 g = mg + vec2(float(i),float(j));
//        vec2 o = hash2( n + g );
//        o = 0.5 + 0.5*sin( u_time + 6.2831*o );
//        vec2 r = g + o - f;
//
//        if( dot(mr-r,mr-r)>0.00001 )
//        md = min( md, dot( 0.5*(mr+r), normalize(r-mr) ) );
//    }


    return vec3( md, mr );
}

float sphere(float t, float k)
{
    float d = 1.0+t*t-t*t*k*k;
    if (d <= 0.0)
    return -1.0;
    float x = (k - sqrt(d))/(1.0 + t*t);
    return asin(x*t);
}


void main() {

    vec2 uv = v_uv - vec2(0.5, 0.5);
    uv *= 3.0;
    float len = length(uv);
    float k = 1.0;
    float len2;

    len2 = sphere(len*k,sqrt(2.0))/sphere(1.0*k,sqrt(2.0));
    uv = uv * len2 * 0.5 / len;
    uv = uv + 0.5;
    //
    vec2 pos = uv;
    float t = u_time/1.0;
    float scale1 = 40.0;
    float scale2 = 20.0;
    float val = 0.0;

    val += sin((pos.x*scale1 + t));
    val += sin((pos.y*scale1 + t)/2.0);
    val += sin((pos.x*scale2 + pos.y*scale2 + sin(t))/2.0);
    val += sin((pos.x*scale2 - pos.y*scale2 + t)/2.0);
    val /= 2.0;

    vec3 c = voronoi(64.0*pos );

    // isolines
   // val += 2.0*sin(t)*c.x*(0.5 + 0.5*sin(64.0*c.x));
    val = (cos(PI*val) + 1.0) * 0.5;

    float glow = 0.020 / (0.015 + distance(len, 1.0));
    vec4 col2 = vec4(0.3, 0.7, 1.0, 0.3);
    frag = step(len, 1.0) *  col2 * val + glow * val;
}
