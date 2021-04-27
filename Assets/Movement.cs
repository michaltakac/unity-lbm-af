using UnityEngine;

public class CameraController : MonoBehaviour {
    private float moveSpeed = 0.5f;
    private float scrollSpeed = 10f;
    private Vector3 moveVector;
    private Vector3 scrollVector;
    float horizontalInput;
    float verticalInput;
    float wheelInput;

    void Start()
    {
        moveVector = new Vector3(0, 0, 0);
        scrollVector = new Vector3(0, 0, 0);
    }

    void Update () {
        horizontalInput = Input.GetAxisRaw("Horizontal");
        verticalInput = Input.GetAxisRaw("Vertical");
        wheelInput = Input.GetAxis("Mouse ScrollWheel");
    }

    void FixedUpdate()
    {
        if (Input.GetAxisRaw("Horizontal") != 0 || verticalInput != 0) {
            moveVector.x = horizontalInput;
            moveVector.z = verticalInput;
            transform.position += moveSpeed * moveVector;
        }

        if (Input.GetAxis("Mouse ScrollWheel") != 0) {
            scrollVector.y = -Input.GetAxis("Mouse ScrollWheel");
            transform.position += scrollSpeed * scrollVector;
        }
    }
}