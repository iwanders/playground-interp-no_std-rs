#ifdef USE_LIB
extern int my_lib_fun();
#endif
int zz[10];
void _start()
{
#ifdef USE_LIB
  my_lib_fun();
#endif
}

