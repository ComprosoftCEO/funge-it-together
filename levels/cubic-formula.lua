function generateTestCase()
  local a = math.random(-9, 9)
  local b = math.random(-9, 9)
  local c = math.random(-9, 9)

  -- (x-a) * (x-b) * (x-c) = x^3 - (a + b + c)x^2 (ab + ac + bc)x - abc
  local inputs = { 1, -(a + b + c), a * b + a * c + b * c, -(a * b * c) }
  local outputs = { a, b, c }
  table.sort(outputs)

  return inputs, outputs
end
