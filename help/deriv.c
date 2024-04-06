typedef struct vec3 { double x, y, z;    } vec3_t;
typedef struct vec4 { double x, y, z, w; } vec4_t;

vec4_t vec4homo  (vec3_t u, double w) { return (vec4_t){u.x * w,   u.y * w,   u.z * w,   w        }; }
vec4_t vec4add   (vec4_t u, vec4_t v) { return (vec4_t){u.x + v.x, u.y + v.y, u.z + v.z, u.w + v.w}; }
vec4_t vec4sub   (vec4_t u, vec4_t v) { return (vec4_t){u.x - v.x, u.y - v.y, u.z - v.z, u.w - v.w}; }
vec4_t vec4mul   (vec4_t u, double s) { return (vec4_t){u.x * s,   u.y * s,   u.z * s,   u.w * s  }; }
vec4_t vec4div   (vec4_t u, double s) { return (vec4_t){u.x / s,   u.y / s,   u.z / s,   u.w / s  }; }
vec3_t vec4trunc (vec4_t u)           { return (vec3_t){u.x,       u.y,       u.z                 }; }
vec3_t vecadd    (vec3_t u, vec3_t v) { return (vec3_t){u.x + v.x, u.y + v.y, u.z + v.z};    }
vec3_t vecsub    (vec3_t u, vec3_t v) { return (vec3_t){u.x - v.x, u.y - v.y, u.z - v.z};    }
vec3_t vecmul    (vec3_t u, double s) { return (vec3_t){u.x * s,   u.y * s,   u.z * s  };    }
vec3_t vecdiv    (vec3_t u, double s) { return (vec3_t){u.x / s,   u.y / s,   u.z / s  };    }

typedef struct pv {
    vec3_t position;
    vec3_t velocity;
} pv_t;

typedef struct nurbs {
    vec3_t P[100];
    double w[100];
    double U[100];
    int    p;
    int    m;
    int    n;
} nurbs_t;

int findspan(double* U, double t, int n, int p) {
    if(t >= U[n]) { return n - 1; }
    if(t <= U[p]) { return p;     }
    int low  = p;
    int high = n;
    int mid  = (low + high) / 2;
    while(t < U[mid] || t >= U[mid+1]) {
        if(t < U[mid]) { high = mid; }
        else           { low  = mid; }
        mid = (low + high) / 2;
    }
    return mid;
}

pv_t nurbs_deboor(double t, nurbs_t* func) {
    vec3_t* P = func->P;
    double* U = func->U;
    double* w = func->w;
    int p     = func->p;
    int m     = func->m;
    int n     = func->n;

    int k = findspan(U, t, n, p);
    vec4_t d[30];
    vec4_t q[30];
    for(int i = 0; i < p + 1; i++) {
        d[i] = vec4homo(P[i+k-p], w[i+k-p]);
        if(!(i < p)) { continue; }
        q[i] = vec4mul(vec4sub(vec4homo(P[i+k-p+1], w[i+k-p+1]), vec4homo(P[i+k-p], w[i+k-p])), p);
        q[i] = vec4div(q[i], U[i+k+1] - U[i+k-p+1]);
    }
    for(int r = 1; r < p + 1; r++) {
        for(int j = p; j > r - 1; j--) {
            double alpha = (t - U[j+k-p]) / (U[j+1+k-r] - U[j+k-p]);
            d[j]  = vec4add(vec4mul(d[j-1], 1.0-alpha), vec4mul(d[j], alpha));
            if(!(r < p && j < p)) { continue; }
            alpha = (t - U[j+k-p+1]) / (U[j+1+k-r] - U[j+k-p+1]);
            q[j]  = vec4add(vec4mul(q[j-1], 1.0-alpha), vec4mul(q[j], alpha));
        }
    }
    pv_t pv;
    pv.position = vecdiv(vec4trunc(d[p]), d[p].w);
    pv.velocity = vecdiv(vecsub(vec4trunc(q[p-1]), vecmul(pv.position, q[p-1].w)), d[p].w);
    return pv;
}

