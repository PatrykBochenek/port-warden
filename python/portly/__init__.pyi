from portly._lib import (
    PortlyError as PortlyError,
)
from portly._lib import (
    PortlyPortError as PortlyPortError,
)
from portly._lib import (
    __version__ as __version__,
)
from portly._lib import (
    find_free as find_free,
)
from portly._lib import (
    get_info as get_info,
)
from portly._lib import (
    is_available as is_available,
)
from portly._lib import (
    kill as kill,
)
from portly._lib import (
    scan as scan,
)
from portly._lib import (
    wait_until_free as wait_until_free,
)

class PortlyPermissionError(PortlyError, PermissionError):
    """Permission denied when inspecting or killing a process.

    Multiple inheritance keeps both ``except PortlyError`` and
    ``except PermissionError`` handling working.
    """

__all__ = [
    "PortlyError",
    "PortlyPermissionError",
    "PortlyPortError",
    "__version__",
    "find_free",
    "get_info",
    "is_available",
    "kill",
    "scan",
    "wait_until_free",
]
