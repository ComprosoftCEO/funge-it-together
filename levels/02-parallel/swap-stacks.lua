function generateTestCase()
  local p0_stack, p1_stack = {}, {}

  for i = 1, math.random(1, 8) do
    p0_stack[i] = math.random(-999, 999)
  end

  for i = 1, math.random(1, 8) do
    p1_stack[i] = math.random(-999, 999)
  end

  return p0_stack, p1_stack, p1_stack, p0_stack
end
