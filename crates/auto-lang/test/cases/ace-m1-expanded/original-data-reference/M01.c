/* PLAN-941 independently authored C11 value reference. Fixed arrays model
 * string values independently; source resource/lifetime obligations have their own gate. */
#include <fcntl.h>
#include <io.h>
#include <stdio.h>
#include <string.h>
typedef struct{int left;int right;} Pair;typedef struct{int flag;Pair p;} Inner;
static Pair pair_new(int left,int right){Pair p={left,right};return p;}
static int left_is(const Pair *p,int value){return p->left==value;}
static int calc(void){Pair p=pair_new(1,1);Pair q=p;q.left=5;if(p.left!=1){return 1;}if(q.left!=5){return 2;}if(!left_is(&p,1)){return 3;}
Inner i={0,p};if(!i.p.right){return 4;}if(!left_is(&i.p,1)){return 5;}Pair n={9,1};if(n.left!=9){return 6;}return 0;}
int main(void){_setmode(_fileno(stdout),_O_BINARY);printf("value:%d\n",calc());return 0;}
