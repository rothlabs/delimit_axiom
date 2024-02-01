# Axiom
Axiom is WebAssembly (WASM) module for converting continuous geometry to discrete geometry. 
Functions like ```axiom.get_mesh({model})``` or  ```axiom.get_polyline({model})``` are available from the JavaScript client. The model is a serializable object that describes geometry and paths with parts like, Circles, Areas, Extrusions, Non-uniform rational B-splines (NURBS), etc. Axiom is designed as the core of the Delimit web app, but it will work just fine for other applications. The largest goal is to enable Delimit to output G-Code for additive and subtractive machines. There was a sucessfuly run with G-Code generation from JavaScript for a 5-Axis 3D printer. However, there is a lot of work ahead to translate that into Rust and improve stability.  

## T-Slot Aluminum Bar Example
```
import {axiom} from 'delimit';

const model = get_model();
const {vector, triangles} = axiom.get_mesh({model});
// do something with "vector" and "triangles" mesh data
// "vector" is a flat array of vertex positions where each triple is x, y, z.
// "triangles" is a flat array of vertex indices where each triple makes a triangle

function get_model(){

    const size = 10;
    const slot_depth = 6.1;
    const slot_width = 3.1;
    const slot_bevel = 0.45;
    const corner_radius = 1.5;
    const corner_position = size - corner_radius + Math.cos(Math.PI/4) * corner_radius;
    const radii = {Vector:[corner_radius, corner_radius]};

    const parts = [
        {MoveTo: {Vector:[0, size - slot_depth]}},
        {LineTo: {Vector:[2.9, size - slot_depth]}},
        {LineTo: {Vector:[6.5, size - 2.5]}},
        {LineTo: {Vector:[6.5, size - 1.5]}},
        {LineTo: {Vector:[slot_width, size - 1.5]}},
        {LineTo: {Vector:[slot_width, size - slot_bevel]}},
        {LineTo: {Vector:[slot_width + slot_bevel, size]}},
        {LineTo: {Vector:[size - corner_radius, size]}},
        {ArcTo:  {to:{Vector:[corner_position, corner_position]}, radii}},
    ];

    const side = {
        Path:{
            parts:[
                {Group:{
                    parts: [{Path:{parts, reverse:true}}], 
                    scale: {Vector:[-1, 1, 1]}, 
                }}, 
                ...parts,
            ]
        }
    };

    const profile = {
        Path:{
            parts:[
                side,
                {Group:{parts:[side], angle:-Math.PI/2}}, 
                {Group:{parts:[side], angle:-Math.PI}}, 
                {Group:{parts:[side], angle:-Math.PI*3/2}},
            ]
        }
    };

    const area = {
        Area:{
            parts:[
                profile, 
                {Circle:{radius: 2.5}},
            ]
        }
    };

    return {
        Extrusion:{
            parts: [area], 
            length: 100,
        }
    }
}
```