function generateTestCase()
  local matrix = {}
  for i = 1, 4 do
    matrix[i] = math.random(-9, 9)
  end

  local determinant = matrix[1] * matrix[4] - matrix[2] * matrix[3]

  local vectors, products = {}, {}
  for i = 1, 8, 2 do
    vectors[i] = math.random(-9, 9)
    vectors[i + 1] = math.random(-9, 9)

    products[i] = matrix[1] * vectors[i] + matrix[2] * vectors[i + 1]
    products[i + 1] = matrix[3] * vectors[i] + matrix[4] * vectors[i + 1]
  end

  return vectors, products, matrix, { determinant }
end
