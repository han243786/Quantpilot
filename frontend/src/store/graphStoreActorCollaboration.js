import { sanitizeDisplayText } from "../utils/errorText";

export const DEFAULT_LOCAL_ACTOR = {
  actor_id: "local_operator",
  display_name: "Local operator"
};

function sanitizeText(value, fallback) {
  return sanitizeDisplayText(value, fallback);
}

export function normalizeActorIdentity(actor, fallback = DEFAULT_LOCAL_ACTOR) {
  const actorId = sanitizeText(actor?.actor_id, fallback.actor_id);
  const displayName = sanitizeText(actor?.display_name, fallback.display_name || actorId);
  return {
    actor_id: actorId || fallback.actor_id,
    display_name: displayName || fallback.display_name
  };
}

export function normalizeCollaborationMetadata(collaboration) {
  return {
    owner:
      collaboration?.owner && typeof collaboration.owner === "object"
        ? normalizeActorIdentity(collaboration.owner)
        : null,
    editors: Array.isArray(collaboration?.editors)
      ? collaboration.editors.map((actor) => normalizeActorIdentity(actor)).filter((actor) => actor.actor_id)
      : [],
    last_saved_by:
      collaboration?.last_saved_by && typeof collaboration.last_saved_by === "object"
        ? normalizeActorIdentity(collaboration.last_saved_by)
        : null,
    last_run_actor:
      collaboration?.last_run_actor && typeof collaboration.last_run_actor === "object"
        ? normalizeActorIdentity(collaboration.last_run_actor)
        : null
  };
}

export function resolveGraphActor(graph) {
  const collaboration = normalizeCollaborationMetadata(graph?.metadata?.collaboration);
  return collaboration.owner || collaboration.editors[0] || DEFAULT_LOCAL_ACTOR;
}

export function withGraphActorMetadata(graph, actor = resolveGraphActor(graph)) {
  const collaboration = normalizeCollaborationMetadata(graph?.metadata?.collaboration);
  if (!collaboration.owner) {
    collaboration.owner = normalizeActorIdentity(actor);
  }
  collaboration.last_saved_by = normalizeActorIdentity(actor);
  return {
    ...graph,
    metadata: {
      ...(graph?.metadata || {}),
      collaboration
    }
  };
}
