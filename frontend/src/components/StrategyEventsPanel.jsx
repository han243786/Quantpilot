import AssetCandlesPanel from "./AssetCandlesPanel";
import { EventFeedSection } from "./EventStreamPanel";

export default function StrategyEventsPanel({
  className = "",
  graph,
  runtime,
  eventTypes,
  eventNodeOptions,
  selectedEventNodeId,
  filteredEvents,
  eventFilters,
  setEventNodeScope,
  setEventTypeFilter,
  setEventSearchTerm,
  setSelectedNode
}) {
  return (
    <div className={`event-events-panel ${className}`.trim()}>
      <AssetCandlesPanel graph={graph} runtime={runtime} />
      <EventFeedSection
        runtime={runtime}
        eventTypes={eventTypes}
        eventNodeOptions={eventNodeOptions}
        selectedEventNodeId={selectedEventNodeId}
        filteredEvents={filteredEvents}
        eventTypeFilter={eventFilters?.eventTypeFilter}
        eventSearchTerm={eventFilters?.eventSearchTerm}
        setEventNodeScope={setEventNodeScope}
        setEventTypeFilter={setEventTypeFilter}
        setEventSearchTerm={setEventSearchTerm}
        setSelectedNode={setSelectedNode}
      />
    </div>
  );
}
