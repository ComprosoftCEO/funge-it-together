function generateTestCase()
  local p0_input, p1_input = {}, {}
  for i = 1, math.random(1, 8) do
    p0_input[i] = math.random(-999, 999)
  end
  for i = 1, math.random(1, 8) do
    p1_input[i] = math.random(-999, 999)
  end

  local odd_output = {}
  for i = 1, #p0_input, 2 do
    odd_output[#odd_output + 1] = p0_input[i]
  end
  for i = 1, #p1_input, 2 do
    odd_output[#odd_output + 1] = p1_input[i]
  end

  local even_output = {}
  for i = 2, #p0_input, 2 do
    even_output[#even_output + 1] = p0_input[i]
  end
  for i = 2, #p1_input, 2 do
    even_output[#even_output + 1] = p1_input[i]
  end

  return p0_input, odd_output, p1_input, even_output
end
