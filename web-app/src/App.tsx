import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { Dashboard } from "@/components/clusters/Dashboard"

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 2,
      refetchOnWindowFocus: false,
    },
  },
})

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <Dashboard />
    </QueryClientProvider>
  )
}
