function generateTestCase()
  local inputs, outputs = {}, {}

  for i = 1, 15 do
    -- Palindrome numbers are pretty uncommon, so generate them with a 50% probability
    if math.random(1, 100) >= 50 then
      inputs[i] = randomPalindrome()
    else
      inputs[i] = math.random(0, 999)
    end

    if isPalindrome(inputs[i]) then
      outputs[#outputs + 1] = inputs[i]
    end
  end

  return inputs, outputs
end

-- Guaranteed to pick a random palindrome number
function randomPalindrome()
  local x = math.random(0, 999)
  while not isPalindrome(x) do
    x = (x + 1) % 1000
  end

  return x
end

function isPalindrome(x)
  -- 1-digit number
  if x < 10 then
    -- By definition: all single-digit numbers are palindromes
    return true
  end

  -- 2-digit number
  if x < 100 then
    local lower = x % 10
    local higher = math.floor(x / 10)

    return lower == higher
  end

  -- 3-digit number
  local lower = x % 10
  local higher = math.floor(x / 100)
  return lower == higher
end
