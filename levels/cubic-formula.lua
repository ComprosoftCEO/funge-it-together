local MAX_X = 9        -- -9 <= x <= 9

local validCubics = {} -- Computed below

function generateTestCase()
  local c = math.random(1, #validCubics)

  local inputs = validCubics[c][1]

  local outputs = uniqueValues(validCubics[c][2])
  table.sort(outputs)

  return inputs, outputs
end

-- a,b,c are the roots
-- (x-a) * (x-b) * (x-c) = x^3 - (a + b + c)x^2 (ab + ac + bc)x - abc
function buildCubic(a, b, c)
  return { 1, -(a + b + c), a * b + a * c + b * c, -(a * b * c) }
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

-- Make sure computing f(x) with Horner's Method won't overflow [-999,999]
function coefficientsTooLarge(poly)
  for x = -MAX_X, MAX_X do
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

-- Build cache of all cubics that don't overflow with Horner's method
for a1 = -MAX_X, MAX_X do
  for a2 = a1, MAX_X do
    for a3 = a2, MAX_X do
      local cubic = buildCubic(a1, a2, a3)
      if not coefficientsTooLarge(cubic) then
        validCubics[#validCubics + 1] = { cubic, { a1, a2, a3 } }
      end
    end
  end
end
