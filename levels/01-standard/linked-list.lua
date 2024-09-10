local NUM_ENTRIES = 10

function generateTestCase()
  -- Generate a random permutation of the numbers 1..10
  local numbersLeft = {}
  for i = 1, NUM_ENTRIES do
    numbersLeft[i] = i
  end

  local linkedList = {}
  for i = 1, #numbersLeft do
    local randomIndex = math.random(1, #numbersLeft)
    linkedList[#linkedList + 1] = table.remove(numbersLeft, randomIndex)
  end

  local randomStart = math.random(1, NUM_ENTRIES)

  -- Find the loop
  local loop = {}
  local current = randomStart
  local visited = {}
  while not visited[current] do
    visited[current] = true
    loop[#loop + 1] = current
    current = linkedList[current]
  end

  -- Input is: random start || linked list
  local input = { randomStart }
  for i = 1, #linkedList do
    input[i + 1] = linkedList[i]
  end

  -- Subtract 1 from everything to make it 0-based index
  for i = 1, #input do
    input[i] = input[i] - 1
  end
  for i = 1, #loop do
    loop[i] = loop[i] - 1
  end

  return input, loop
end
