test({
  minify: true,
  compress: {
    unused: true,
    dead_code: true,
    reduce_vars: true,
    sequences: false // for better readability of output
  }
}); 
