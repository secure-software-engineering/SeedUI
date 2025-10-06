#include <stdio.h>
#include "inner/foo.h"

int main(int argc, char **argv) {
  int a, b;
  printf("Enter something:\n");
  scanf("%d", &a);
  printf("Enter something else: \n");
  scanf("%d", &b);
  if (a > 0 && a < 50) {
    if (b > 0 && b < 50) {
      foo(a);
      foo(b);
      test(a, b);
    }
  }
}