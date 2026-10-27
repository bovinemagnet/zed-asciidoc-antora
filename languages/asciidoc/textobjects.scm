; Vim mode selects a section with `ac` / `ic`, a delimited block with `af` / `if`,
; and a comment with `gc`. Inside a section is everything below its title; inside a
; block is everything between its delimiter lines.

(section) @class.around

(section
  [
    (section_block)
    (section)
    (line_comment)
    (block_comment)
  ]+ @class.inside)

[
  (delimited_block)
  (sidebar_block)
  (open_block)
  (quoted_block)
  (quoted_md_block)
  (listing_block)
  (literal_block)
  (passthrough_block)
  (table_block)
  (ntable_block)
  (csv_table_block)
  (dsv_table_block)
] @function.around

(delimited_block
  (section_block)+ @function.inside)

(sidebar_block
  (section_block)+ @function.inside)

(open_block
  (section_block)+ @function.inside)

(quoted_block
  [
    (line)
    (quoted_block)
  ]+ @function.inside)

(quoted_md_block
  (line)+ @function.inside)

(listing_block
  (listing_block_body) @function.inside)

(literal_block
  (literal_block_body) @function.inside)

(passthrough_block
  (listing_block_body) @function.inside)

(table_block
  [
    (table_header_row)
    (table_cell)
    (ntable_block)
  ]+ @function.inside)

(ntable_block
  (ntable_cell)+ @function.inside)

(csv_table_block
  (csv_record)+ @function.inside)

(dsv_table_block
  (dsv_record)+ @function.inside)

(line_comment) @comment.around

(line_comment
  (body) @comment.inside)

(block_comment) @comment.around

(block_comment
  (body) @comment.inside)
