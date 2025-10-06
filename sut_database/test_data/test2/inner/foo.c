#include "foo.h"

void foo(int a) { printf("got %d\n", a); }

void test(int a, int b) {
  while (a > 0) {
    if (b == 0) {
      printf("b<a\n");
      return;
    }
    a--;
    b--;
  }
  if (b == 0) {
    printf("b==a\n");
    return;
  }
  printf("b>a\n");
  return;
}