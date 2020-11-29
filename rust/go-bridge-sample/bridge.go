package main

// typedef const char *const_string_t;
import "C"

import (
	"cuelang.org/go/cue"
)

//export cue_export
func cue_export(filename C.const_string_t, code C.const_string_t, e *C.int) *C.char {
	f := C.GoString(filename)
	c := C.GoString(code)
	r := cue.Runtime{}
	instance, err := r.Compile(f, c)
	if err != nil {
		*e = 1
		return C.CString(err.Error())
	}
	json, err := instance.Value().MarshalJSON()
	if err != nil {
		*e = 1
		return C.CString(err.Error())
	}
	*e = 0
	return C.CString(string(json))
}

func main() {
}
