float d0 = 2. * k1u / k0k1 / k1r1;
float d1 = (2. * (k0 * ur1 + k1 * k2u + u * r1k2)) / (k0k1 * k0k2 * k1r1);
float d2 = (2. * uk0) / (k0k1 * k0k2);
return float[8](0., p0/sum, p1/sum, p2/sum, 0., d0, d1, d2);