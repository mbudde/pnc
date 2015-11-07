" Vim syntax file
" Language: PNC
" Maintainer: Michael Budde
" Latest Revision: 7 November 2015

if exists("b:current_syntax")
  finish
endif


syn keyword pncBuiltin add alias sub mul div mod swap dup print stdin map fold repeat pop def roll3 len sum

syn match pncNumber '\(^\|\s\)\zs[+-]\?\d\+\(\.\d\+\)\?'

syn match pncQuote '\(^\|\s\),'

syn region pncComment start="#" end="$" contains=@Spell

syn match pncPunct '\({\|}\|\[\[\|\]\]\|\[\|\]\)'

hi def link pncBuiltin Keyword
hi def link pncNumber  Number
hi def link pncQuote   Special
hi def link pncComment Comment
hi def link pncPunct   PreProc
