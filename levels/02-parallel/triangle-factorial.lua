local MAX_TRIANGLE = 44
local MAX_FACTORIAL = 6

function generateTestCase()
  local triangle_inputs, triangle_outputs = {}, {}
  local factorial_inputs, factorial_outputs = {}, {}

  for i = 1, 8 do
    triangle_inputs[i] = math.random(0, MAX_TRIANGLE)
    triangle_outputs[i] = triangle(triangle_inputs[i])

    factorial_inputs[i] = math.random(0, MAX_FACTORIAL)
    factorial_outputs[i] = factorial(factorial_inputs[i])
  end

  return triangle_inputs, factorial_outputs, factorial_inputs, triangle_outputs
end

function triangle(x)
  local sum = 0
  for i = 1, x do
    sum = sum + i
  end
  return sum
end

function factorial(x)
  if x < 1 then
    return 1
  else
    return x * factorial(x - 1)
  end
end
