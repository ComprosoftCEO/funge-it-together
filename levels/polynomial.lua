function generateTestCase()
  local outputs = {}

  -- Generate random polynomial with 2 to 7 coefficients
  local polynomial = randomCoefficient()
  outputs[#outputs + 1] = polynomial[2]
  for i = 1, math.random(1, 6) do
    local newPolynomial = multiplyPolynomials(polynomial, randomCoefficient())
    while coefficientsTooLarge(newPolynomial) do
      newPolynomial = multiplyPolynomials(polynomial, randomCoefficient())
    end

    polynomial = newPolynomial
    outputs[#outputs + 1] = polynomial[2]
  end

  outputs = uniqueValues(outputs)
  table.sort(outputs)

  return polynomial, outputs
end

-- Return (x - a), where a is a random number in [-9, 9]
function randomCoefficient()
  return { 1, math.random(-9, 9) }
end

-- Multiply two polynomials in vector form and return a new polynomial
--
-- Example:
--   (x^2 + 2x - 6) * (x^3 + x + 1) = x^5 + 2x^4 - 5x^3 + 3x^2 - 4x - 6
--   {2, 2, -6}     * {1, 0, 1, 1}  = {1, 2, 5, 3, -4, -6}
function multiplyPolynomials(a, b)
  local newPolynomial = {}
  for i = 1, #a do
    for j = 1, #b do
      local index = (i - 1) + (j - 1) + 1 -- Coefficient of the polynomial
      newPolynomial[index] = (newPolynomial[index] or 0) + a[i] * b[j]
    end
  end

  return newPolynomial
end

-- Make sure all coefficients are small enough to multiply without overflowing
function coefficientsTooLarge(x)
  for i = 1, #x do
    if x[i] < -99 or x[i] > 99 then
      return true
    end
  end

  return false
end

-- Get only the unique values in the array
function uniqueValues(x)
  local uniqueValues = {}
  for i = 1, #x do
    uniqueValues[x[i]] = true
  end

  local res = {}
  for k in pairs(uniqueValues) do
    res[#res + 1] = k
  end

  return res
end
