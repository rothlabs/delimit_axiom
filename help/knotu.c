/*  Subroutine to generate a B-spline uniform (periodic) knot vector.

    c            = order of the basis function
    n            = the number of defining polygon vertices
    nplus2       = index of x() for the first occurence of the maximum knot vector value
    nplusc       = maximum value of the knot vector -- $n + c$
    x[]          = array containing the knot vector
*/

#include	<stdio.h>

knotu(n,c,x)

int n,c;
int x[];

{
    int nplusc,nplus2,i;

	nplusc = n + c;
	nplus2 = n + 2;

	x[1] = 0;
	for (i = 2; i <= nplusc; i++){
	    x[i] = i-1;
	}
}
