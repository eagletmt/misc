#include "wrapper.h"

void mrb_miam2tf_gem_init(mrb_state* _mrb) {
}

void mrb_miam2tf_gem_final(mrb_state* _mrb) {
}

mrb_aspec wrapper_mrb_args_req(mrb_int n) {
  return MRB_ARGS_REQ(n);
}

mrb_int wrapper_rarray_len(mrb_value ary) {
  return RARRAY_LEN(ary);
}

mrb_int wrapper_mrb_nil_p(mrb_value o) {
  return mrb_nil_p(o);
}

struct RClass* wrapper_e_runtime_error(mrb_state *mrb) {
  return E_RUNTIME_ERROR;
}

mrb_value wrapper_mrb_nil_value(void) {
  return mrb_nil_value();
}

mrb_int wrapper_mrb_integer(mrb_value n) { return mrb_integer(n); }
