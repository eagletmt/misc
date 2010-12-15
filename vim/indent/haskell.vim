" indentexpr for Haskell
" Author:  eagletmt <eagletmt@gmail.com>
" Last Modified: 15 Dec 2010

if exists('b:did_indent') && b:did_indent
  finish
endif
let b:did_indent = 1

setlocal expandtab
setlocal indentexpr=haskell#indent()
setlocal indentkeys=0{,!^F,o,O,0=else,0=in,0\|
inoremap <buffer> <expr> <C-d> <SID>unindent()
let b:undo_indent = 'setlocal expandtab< indentexpr< indentkeys< | iunmap <buffer> <C-d>'

function! haskell#indent()"{{{
  if getline('.') =~# '^\s*$'
    return s:on_newline()
  endif

  let l:prev_lnum = line('.')-1
  let l:prev_line = getline(l:prev_lnum)
  let l:current_lnum = line('.')
  let l:current_line = getline('.')

  " if expr
  if l:current_line =~# '^\s*\<else\>'
    let l:lnum = l:prev_lnum
    while l:lnum >= 1
      let l:line = getline(l:lnum)
      if l:line =~# '^\s*\<then\>'
        " adjust to then
        return indent(l:lnum)
      elseif l:line =~# '\<if\>'
        return match(l:line, '\(\<else\>\s\+\)\?\<if\>')
      endif
      let l:lnum -= 1
    endwhile
  endif

  " data / newtype
  if l:current_line =~# '^\s*{' && l:prev_line =~# '\<data\>\|\<newtype\>'
    return indent(l:prev_lnum) + &l:shiftwidth
  endif

  " let expr
  if l:current_line =~# '^\s*in'
    " unindent
    let l:lnum = l:prev_lnum
    while l:lnum >= 1
      if getline(l:lnum) =~# '\<let\>'
        let l:pos = match(getline(l:lnum), '\<let\>')
        return l:pos
      end
      let l:lnum -= 1
    endwhile
    " ???
    return indent(l:current_lnum) - &l:shiftwidth
  endif

  " guard or data-ctor
  if l:current_line =~# '^\s*|'
    if l:prev_line =~# '^\s*|'
      " keep previous guard layout
      return indent(l:prev_lnum)
    elseif l:prev_line =~# '^\s*\<data\>'
      " data
      return match(l:prev_line, '=')
    else
      " maybe guard start
      return indent(l:prev_lnum) + &l:shiftwidth
    endif
  endif

  " keep
  return indent(l:current_lnum)
endfunction"}}}

function! s:on_newline()"{{{
  let l:prev_lnum = line('.')-1
  let l:prev_line = getline(l:prev_lnum)

  " case expr
  if l:prev_line =~# '\<of\>\s*$'
    let l:lnum = l:prev_lnum
    while l:lnum >= 1
      if getline(l:lnum) =~# '\<case\>'
        return match(getline(l:lnum), '\<case\>') + &l:shiftwidth
      endif
      let l:lnum -= 1
    endwhile
  endif

  " if expr
  if l:prev_line =~# '\(\<else\>\|\<then\>\)\s*$'
    let l:lnum = l:prev_lnum
    while l:lnum >= 1
      if getline(l:lnum) =~# '\<if\>'
        return match(getline(l:lnum), '\(\<else\>\s\+\)\?\<if\>') + &l:shiftwidth
      endif
      let l:lnum -= 1
    endwhile
  endif

  " do expr
  if l:prev_line =~# '\<do\>\s*$'
    return indent(l:prev_lnum) + &l:shiftwidth
  elseif l:prev_line =~# '\<do\>\s\+[^{]*$'
    return matchend(l:prev_line, '\<do\>\s\+')
  endif

  " let expr
  " XXX: doesn't recognize let statement
  if l:prev_line =~# '^\s*\<let\>'
    let l:layout = matchend(l:prev_line, '^\s*\<let\>\s\+')
    return l:layout
  endif

  " where
  if l:prev_line =~# '\<where\>'
    if l:prev_line =~# '^\s*\<where\>'
      return indent(l:prev_lnum) + &l:shiftwidth
    endif

    let l:lnum = l:prev_lnum
    while l:lnum >= 1
      let l:line = getline(l:lnum)
      if l:line =~# '^\s*\(\<class\>\|\<instance\>\)'
        return indent(l:lnum) + &l:shiftwidth
      elseif l:line =~# '^\s*\(\w\|$\)'
        break
      endif
      let l:lnum -= 1
    endwhile
  endif

  " deriving
  if l:prev_line =~# '\<deriving\>'
    " reset
    return 0
  endif

  " function definition
  if l:prev_line =~# '=\s*$'
    return indent(l:prev_lnum) + &l:shiftwidth
  endif

  " begin list literal
  if l:prev_line =~# '\[[^]]*$'
    return match(l:prev_line, '\[[^]]*$')
  endif

  " end list literal
  if l:prev_line =~# '\]\s*$'
    return haskell#unindent_layout(l:prev_lnum)
  endif

  " case alts or lambda
  if l:prev_line =~# '->\s*$'
    return indent(l:prev_lnum) + &l:shiftwidth
  endif

  " keep layout
  return indent(l:prev_lnum)
endfunction"}}}

function! s:unindent()"{{{
  let l:count = haskell#unindent_layout(line('.'))
  if l:count >= 0
    let l:w = col('.') - indent('.') - 1
    return repeat("\<Left>", l:w) . repeat("\<BS>", l:count) . repeat("\<Right>", l:w)
  else
    return "\<C-d>"
  endif
endfunction"}}}

function! haskell#unindent_layout(lnum)"{{{
  let l:lnum = a:lnum
  let l:layout = indent(a:lnum)
  while l:lnum >= 1
    echom 'indent=' indent(l:lnum) 'for lnum =' l:lnum
    if indent(l:lnum) < l:layout
      return indent(l:lnum)
    endif
    let l:lnum -= 1
  endwhile
  return -1
endfunction"}}}
