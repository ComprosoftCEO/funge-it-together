function generateTestCase()
  local a = math.random(-9, 9)
  local b = math.random(-9, 9)

  -- (x-a) * (x-b) = x^2 - (a+b)x + ab
  local inputs = { 1, -(a + b), a * b }

  local outputs = { math.min(a, b), math.max(a, b) }
  if a == b then
    outputs = { a } -- Special case: only output duplicates once
  end

  return inputs, outputs
end
