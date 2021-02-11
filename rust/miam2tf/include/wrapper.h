#include <mruby.h>
#include <mruby/array.h>
#include <mruby/boxing_word.h>
#include <mruby/compile.h>
#include <mruby/hash.h>
#include <mruby/string.h>
#include <mruby/variable.h>

mrb_aspec wrapper_mrb_args_req(mrb_int n);
mrb_int wrapper_rarray_len(mrb_value ary);
mrb_int wrapper_mrb_nil_p(mrb_value o);
struct RClass* wrapper_e_runtime_error(mrb_state *mrb);
mrb_value wrapper_mrb_nil_value(void);
mrb_int wrapper_mrb_integer(mrb_value n);
