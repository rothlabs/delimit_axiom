/*  Subroutine to generate rational B-spline basis functions--open knot vector

	C code for An Introduction to NURBS
	by David F. Rogers. Copyright (C) 2000 David F. Rogers,
	All rights reserved.
	
	Name: rbais
	Language: C
	Subroutines called: none
	Book reference: Chapter 4, Sec. 4. , p 296

	c        = order of the B-spline basis function
    d        = first term of the basis function recursion relation
    e        = second term of the basis function recursion relation
	h[]	     = array containing the homogeneous weights
    npts     = number of defining polygon vertices
    nplusc   = constant -- npts + c -- maximum number of knot values
    r[]      = array containing the rationalbasis functions
               r[1] contains the basis function associated with B1 etc.
    t        = parameter value
    temp[]   = temporary array
    x[]      = knot vector
*/	

#include 	<stdio.h>

rbasis(c,t,npts,x,h,r)

int c,npts;
float t;
int x[];
float h[];
float r[];

{
	int nplusc;
	int i,j,k;
	float d,e;
	float sum;
	float temp[36];

	nplusc = npts + c;

/*		printf("knot vector is \n");
		for (i = 1; i <= nplusc; i++){
			printf(" %d %d \n", i,x[i]);
		}
		printf("t is %f \n", t);
*/

/* calculate the first order nonrational basis functions n[i]	*/

	for (i = 1; i<= nplusc-1; i++){ // for (i = 1; i < nplusc; i++){
    	if (( t >= x[i]) && (t < x[i+1]))
			temp[i] = 1;
	    else
			temp[i] = 0;
	}

/* calculate the higher order nonrational basis functions */

	for (k = 2; k <= c; k++){ // order (could be degree instead)
    	for (i = 1; i <= nplusc-k; i++){ // i'th control point
        	if (temp[i] != 0)    /* f i n     if the lower order basis function is zero skip the calculation */
           		d = ((t-x[i])*temp[i])/(x[i+k-1]-x[i]); // could be i+k instead of i+k-1 if second part has i+k+1 and k was degree instead of order
	        else
				d = 0;

    	    if (temp[i+1] != 0)     /* g i+1 n   if the lower order basis function is zero skip the calculation */
        		e = ((x[i+k]-t)*temp[i+1])/(x[i+k]-x[i+1]);
	        else
    			e = 0;

    	    temp[i] = d + e;
		}
	}

	if (t == (float)x[nplusc]){		/*    pick up last point	*/
 		temp[npts] = 1;
	}
/*
	printf("Nonrational basis functions are \n");
	for (i=1; i<= npts; i++){
		printf("%f ", temp[i]);
	}
	printf("\n");
*/
/* calculate sum for denominator of rational basis functions */

	sum = 0.;
	for (i = 1; i <= npts; i++){
		    sum = sum + temp[i]*h[i];
	}

/* form rational basis functions and put in r vector */

	for (i = 1; i <= npts; i++){
    	if (sum != 0){
        	r[i] = (temp[i]*h[i])/(sum);}
		else
			r[i] = 0;
	}
}
