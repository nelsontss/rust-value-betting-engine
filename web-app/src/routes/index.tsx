import { createRoute, useNavigate, useSearch } from "@tanstack/react-router"
import { rootRoute } from "./__root"
import { Dashboard } from "@/components/clusters/Dashboard"

export const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  validateSearch: (search: Record<string, unknown>) => ({
    ...(typeof search.cluster === "string" && search.cluster !== ""
      ? { cluster: search.cluster }
      : {}),
    ...(typeof search.market === "string" && search.market !== ""
      ? { market: search.market }
      : {}),
  }),
  component: IndexPage,
})

function IndexPage() {
  const navigate = useNavigate({ from: indexRoute.id })
  const search = useSearch({ from: indexRoute.id })
  const cluster = search.cluster ?? ""
  const market = search.market ?? ""

  return (
    <Dashboard
      clusterId={cluster}
      marketKey={market}
      onSelectCluster={(id) =>
        navigate({
          search: (prev) => {
            const next: Record<string, string> = { ...prev, cluster: id }
            delete next.market
            return next
          },
        })
      }
      onSelectMarket={(key) =>
        navigate({
          search: (prev) => ({ ...prev, market: key }),
        })
      }
    />
  )
}
