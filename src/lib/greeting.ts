export function greetingForHour(hour: number) {
  if (hour >= 5 && hour < 12) return 'Good morning';
  if (hour >= 12 && hour < 17) return 'Good afternoon';
  return 'Good evening';
}

export function personalisedGreeting(hour: number, name: string) {
  const greeting = greetingForHour(hour);
  return name.trim() ? `${greeting}, ${name.trim()}` : greeting;
}
