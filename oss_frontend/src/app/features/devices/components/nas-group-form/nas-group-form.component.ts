import { Component, OnInit } from '@angular/core';
import { FormBuilder, FormGroup, Validators } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { NasService } from '../../../../shared/services/nas.service';
import { NasGroup } from '../../../../shared/models/nas.model';

@Component({
  selector: 'app-nas-group-form',
  templateUrl: './nas-group-form.component.html',
  styleUrls: ['./nas-group-form.component.scss']
})
export class NasGroupFormComponent implements OnInit {
  groupForm: FormGroup;
  isEditMode = false;
  groupId: number | null = null;
  loading = false;
  submitting = false;
  error = '';
  parentGroups: NasGroup[] = [];
  filteredParentGroups: NasGroup[] = [];

  constructor(
    private fb: FormBuilder,
    private nasService: NasService,
    private route: ActivatedRoute,
    private router: Router
  ) {
    this.groupForm = this.fb.group({
      name: ['', [Validators.required, Validators.maxLength(255)]],
      description: [''],
      parent_id: [null],
    });
  }

  ngOnInit(): void {
    this.loading = true;

    // Load all NAS groups for parent selection
    this.nasService.getNasGroupTree()
      .subscribe({
        next: (groups) => {
          this.parentGroups = groups;
          this.filteredParentGroups = [...groups];
          
          // Check if we're in edit mode
          const id = this.route.snapshot.paramMap.get('id');
          if (id && id !== 'new') {
            this.isEditMode = true;
            this.groupId = +id;
            this.loadGroupDetails(+id);
          } else {
            this.loading = false;
          }
        },
        error: (err) => {
          this.error = 'Failed to load NAS groups. Please try again later.';
          console.error('Error loading NAS groups:', err);
          this.loading = false;
        }
      });
  }

  loadGroupDetails(id: number): void {
    this.nasService.getNasGroupById(id)
      .subscribe({
        next: (group) => {
          // Populate the form with group details
          this.groupForm.patchValue({
            name: group.name,
            description: group.description,
            parent: group.parent_id,
            is_active: group.is_active
          });
          
          // Filter out the current group and its children from parent options
          this.filterParentOptions(id);
          
          this.loading = false;
        },
        error: (err) => {
          this.error = 'Failed to load group details. Please try again later.';
          console.error('Error loading group details:', err);
          this.loading = false;
        }
      });
  }

  filterParentOptions(currentGroupId: number): void {
    if (!this.parentGroups || !Array.isArray(this.parentGroups)) {
      this.filteredParentGroups = [];
      return;
    }

    // Filter out the current group and its children
    this.filteredParentGroups = this.parentGroups.filter(group => {
      // Don't include the current group
      if (group.id === currentGroupId) {
        return false;
      }

      // Check if the group is a child of the current group
      let parent = group.parent;
      while (parent) {
        if (parent.id === currentGroupId) {
          return false;
        }
        parent = parent.parent;
      }

      return true;
    });
  }

  onSubmit(): void {
    if (this.groupForm.invalid) {
      return;
    }

    this.submitting = true;
    const formData = this.groupForm.value;

    const request = this.isEditMode
      ? this.nasService.updateNasGroup(this.groupId!, formData)
      : this.nasService.createNasGroup(formData);

    request.subscribe({
      next: () => {
        this.router.navigate(['/devices/groups']);
      },
      error: (err) => {
        this.error = this.isEditMode
          ? 'Failed to update NAS group. Please try again later.'
          : 'Failed to create NAS group. Please try again later.';
        console.error('Error saving NAS group:', err);
        this.submitting = false;
      }
    });
  }

  cancel(): void {
    this.router.navigate(['/devices/groups']);
  }
}