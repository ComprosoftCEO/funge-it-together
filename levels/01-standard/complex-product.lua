function generateTestCase()
  local inputs = {}

  local product = { 1, 0 }
  for i = 1, 2 * math.random(1, 4) + 1, 2 do
    local newNumber, newProduct
    repeat
      newNumber = randomComplex()
      newProduct = multiplyComplex(product, newNumber)
    until newProduct[1] >= -999 and newProduct[1] <= 999
      and newProduct[2] >= -999 and newProduct[2] <= 999
      and not (newProduct[1] == 0 and newProduct[2] == 0)

    inputs[i] = newNumber[1]
    inputs[i + 1] = newNumber[2]
    product = newProduct
  end

  return inputs, product
end

function randomComplex()
  return { math.random(-9, 9), math.random(-9, 9) }
end

function multiplyComplex(one, two)
  local real = one[1] * two[1] - one[2] * two[2]
  local imaginary = (one[1] + one[2]) * (two[1] + two[2]) - one[1] * two[1] - one[2] * two[2]
  return { real, imaginary }
end
