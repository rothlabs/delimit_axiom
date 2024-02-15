fn get_line_intersection(p1: Vec2, p2: Vec2, p3: Vec2, p4: Vec2) -> Option<Vec2> {
    if p1.distance(p2) < EPSILON {
        return None;
    }
    if p3.distance(p4) < EPSILON {
        return  None;
    }
    let t = ((p1.x - p3.x)*(p3.y - p4.y) - (p1.y - p3.y)*(p3.x - p4.x)) 
        / ((p1.x - p2.x)*(p3.y - p4.y) - (p1.y - p2.y)*(p3.x - p4.x));
    let u = - ((p1.x - p2.x)*(p1.y - p3.y) - (p1.y - p2.y)*(p1.x - p3.x))
        / ((p1.x - p2.x)*(p3.y - p4.y) - (p1.y - p2.y)*(p3.x - p4.x));
    if 0. > t || t > 1. || 0. > u || u > 1. {
        return None;
    }
    if (EPSILON > t || t+EPSILON > 1.) && (EPSILON > u || u+EPSILON > 1.) {
        return None;
    }
    let x = p3.x + u*(p4.x - p3.x);
    let y = p3.y + u*(p4.y - p3.y);
    if x.is_nan() || y.is_nan() {
        return None;
    }
    Some(vec2(x, y))
}