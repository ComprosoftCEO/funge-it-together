local MIN_DEGREE = 2
local MAX_DEGREE = 5

-- Range: -X to X
-- Could probably be a function but I don't care
local DEGREE_X_RANGE = {
  [2] = 9,
  [3] = 7,
  [4] = 5,
  [5] = 3,
}

local validPolynomials = {} -- computed below

function generateTestCase()
  local randomDegree = math.random(MIN_DEGREE, MAX_DEGREE)
  local randomIndex = math.random(1, #validPolynomials[randomDegree]) -- Guaranteed to be non-empty

  local coefficients = validPolynomials[randomDegree][randomIndex][1]

  local outputs = uniqueValues(validPolynomials[randomDegree][randomIndex][2])
  table.sort(outputs)

  return {}, outputs, coefficients, {}
end

-- Build a polynomial from a list of 1 or more roots
function buildPolynomial(roots)
  local polynomial = { 1, -roots[1] }
  for i = 2, #roots do
    polynomial = multiplyPolynomials(polynomial, { 1, -roots[i] })
  end

  return polynomial
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

-- Create an iterator over all polynomials of a given degree
--   n is the polynomial degree
--   x varies from [minX, maxX]
function allPolynomials(n, minX, maxX)
  local function gen(tmpN, arr)
    if tmpN == 0 then
      coroutine.yield(buildPolynomial(arr), { table.unpack(arr) })
    else
      local curIndex = n - tmpN
      for x = (arr[curIndex] or minX), maxX do
        arr[curIndex + 1] = x
        gen(tmpN - 1, arr)
      end
    end
  end

  return coroutine.wrap(function() gen(n, {}) end)
end

-- Make sure computing f(x) with Horner's Method won't overflow [-999,999]
function coefficientsTooLarge(poly, min, max)
  for x = min, max do
    local acc = poly[1]
    for i = 2, #poly do
      acc = acc * x -- Multiply
      if acc < -999 or acc > 999 then
        return true
      end

      acc = acc + poly[i] -- Add
      if acc < -999 or acc > 999 then
        return true
      end
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

-- Build cache of all polynomials that don't overflow with Horner's method
for degree = MIN_DEGREE, MAX_DEGREE do
  local validDegreePolynomials = {}
  local degreeMin, degreeMax = -DEGREE_X_RANGE[degree], DEGREE_X_RANGE[degree]

  for polynomial, roots in allPolynomials(degree, degreeMin, degreeMax) do
    if not coefficientsTooLarge(polynomial, degreeMin, degreeMax) then
      validDegreePolynomials[#validDegreePolynomials + 1] = { polynomial, roots }
    end
  end

  validPolynomials[degree] = validDegreePolynomials
end
