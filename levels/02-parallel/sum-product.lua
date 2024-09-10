function generateTestCase()
  local xs, ys = {}, {}
  local sums, products = {}, {}

  for i = 1, 8 do
    xs[i] = math.random(-9, 9)
    ys[i] = math.random(-9, 9)
    sums[i] = xs[i] + ys[i]
    products[i] = xs[i] * ys[i]
  end

  return xs, sums, ys, products
end
